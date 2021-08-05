use crate::ffi::apple as ffi;
// use std::collections::HashMap;
use crate::browse::{Service, ServiceEventType};
use crate::ffi::apple::kDNSServiceErr_NoError;
use crate::ServiceBrowserBuilder;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::io::{Error as IoError, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::raw::c_char;
use std::ptr;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;
use thiserror::Error;

impl From<ffi::DNSServiceFlags> for ServiceEventType {
    fn from(flags: ffi::DNSServiceFlags) -> Self {
        if flags & ffi::kDNSServiceFlagsAdd as u32 != 0 {
            ServiceEventType::Added
        } else {
            ServiceEventType::Removed
        }
    }
}

/// Common error for DNS-SD service
#[derive(Debug, Error)]
pub enum BrowseError {
    /// Invalid input string
    #[error("Invalid string argument, must be C string compatible")]
    InvalidString,
    /// Unexpected invalid strings from C API
    #[error("DNSSD API returned invalid UTF-8 string")]
    InternalInvalidString,
    /// Error from DNSSD service
    #[error("DNSSD Error: {0}")]
    ServiceError(i32),
    /// IO error
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
}
/// Apple based DNS-SD result type
pub type Result<T, E = BrowseError> = std::result::Result<T, E>;

unsafe extern "C" fn browse_callback(
    _sd_ref: ffi::DNSServiceRef,
    flags: ffi::DNSServiceFlags,
    interface_index: u32,
    error_code: ffi::DNSServiceErrorType,
    service_name: *const c_char,
    regtype: *const c_char,
    reply_domain: *const c_char,
    context: *mut c_void,
) {
    if !context.is_null() {
        let tx_ptr: *mut SyncSender<Result<DiscoveredService>> = context as _;
        let tx = &*tx_ptr;

        // shouldn't need any other args if there's an error
        if error_code != 0 {
            match tx.try_send(Err(BrowseError::ServiceError(error_code))) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error sending service notification on channel: {:?}", e);
                }
            }
            return;
        }

        // build Strings from c_char
        let process = || -> Result<(String, String, String)> {
            let c_str: &CStr = CStr::from_ptr(service_name);
            let service_name: &str = c_str
                .to_str()
                .map_err(|_| BrowseError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(regtype);
            let regtype: &str = c_str
                .to_str()
                .map_err(|_| BrowseError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(reply_domain);
            let reply_domain: &str = c_str
                .to_str()
                .map_err(|_| BrowseError::InternalInvalidString)?;
            Ok((
                service_name.to_owned(),
                regtype.to_owned(),
                reply_domain.to_owned(),
            ))
        };
        match process() {
            Ok((name, regtype, domain)) => {
                let mut service = DiscoveredService {
                    name,
                    regtype,
                    interface_index,
                    domain,
                    event_type: flags.into(),
                };
                trace!("Informing of discovered service: {:?}", service);
                match tx.try_send(Ok(service)) {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error sending service notification on channel: {:?}", e);
                    }
                }
            }
            Err(e) => match tx.try_send(Err(e)) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error sending service notification on channel: {:?}", e);
                }
            },
        }
    }
}

/// Encapsulates information about a service
#[derive(Debug)]
pub struct DiscoveredService {
    /// Name of service, usually a user friendly name
    pub name: String,
    /// Registration type, i.e. _http._tcp.
    pub regtype: String,
    /// Interface index (unsure what this is for)
    pub interface_index: u32,
    /// Domain service is on, typically local.
    pub domain: String,
    /// Whether this service is being added or not
    pub event_type: ServiceEventType,
}

fn service_from_resolved(discovered: DiscoveredService, resolved: Vec<ResolvedService>) -> Service {
    if resolved.len() > 1 {
        warn!("We resolved > 1 services, unsupported. using first");
    }
    let (port, hostname, txt_record) = match resolved.into_iter().next() {
        Some(resolved) => (resolved.port, resolved.hostname, resolved.txt_record),
        None => (0, "".to_string(), None),
    };
    Service {
        name: discovered.name,
        domain: discovered.domain,
        regtype: discovered.regtype,
        interface_index: Some(discovered.interface_index),
        event_type: discovered.event_type,
        hostname,
        port,
        txt_record,
    }
}

fn resolver_thread(rx: Receiver<Result<DiscoveredService>>, tx: SyncSender<Result<Service>>) {
    std::thread::Builder::new()
        .name("astro-dnssd: resolver".into())
        .spawn(move || loop {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(service)) => {
                    trace!("Got new service: {:?}, resolving...", service);
                    match service.resolve() {
                        Ok(resolved) => {
                            trace!("Resolved: {:?}", resolved);
                            let service = service_from_resolved(service, resolved);
                            match tx.send(Ok(service)) {
                                Ok(_) => {}
                                Err(_e) => {
                                    error!("Error sending resolved service, disconnected channel, exiting thread");
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error resolving: {:?}", e);
                            if let Err(_e) = tx.send(Err(e)) {
                                error!("Error sending resolved service, disconnected channel, exiting thread");
                                break;
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    if let Err(_e) = tx.send(Err(e)) {
                        error!("Error sending resolved service, disconnected channel, exiting thread");
                        break;
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    error!("Resolver channel disconnected, exiting thread");
                    break;
                }
            }
        }).expect("Failed to start resolver thread");
}

/// Main service browser, calls callback upon discovery of service
pub struct ServiceBrowser {
    // /// Type to search for, i.e. ._http._tcp.
    // pub regtype: String,
    // /// Domain to search in, default is .local
    // pub domain: Option<String>,
    raw: ffi::DNSServiceRef,
    rx: Receiver<Result<Service>>,
}

impl ServiceBrowser {
    /// Returns socket to mDNS service, use with select()
    pub fn socket(&self) -> i32 {
        unsafe { ffi::DNSServiceRefSockFD(self.raw) }
    }

    /// Processes a reply from mDNS service, blocking until there is one
    fn process_result(&self) -> ffi::DNSServiceErrorType {
        // shouldn't get here but to be safe for now
        if self.raw.is_null() {
            return ffi::kDNSServiceErr_Invalid;
        }
        unsafe { ffi::DNSServiceProcessResult(self.raw) }
    }

    /// returns true if the socket has data and process_result() should be called
    fn has_data(&self, timeout: Duration) -> Result<bool> {
        let socket = unsafe { ffi::DNSServiceRefSockFD(self.raw) } as _;
        let r = crate::non_blocking::socket_is_ready(socket, timeout)?;
        Ok(r)
    }

    /// Starts browser with type & optional domain
    fn start(regtype: String, domain: Option<String>) -> Result<Self> {
        unsafe {
            let c_domain: Option<CString>;
            if let Some(d) = &domain {
                c_domain = Some(CString::new(d.as_str()).map_err(|_| BrowseError::InvalidString)?);
            } else {
                c_domain = None;
            }
            let service_type =
                CString::new(regtype.as_str()).map_err(|_| BrowseError::InvalidString)?;
            let (tx, rx) = sync_channel::<Result<DiscoveredService>>(10);
            let tx = Box::into_raw(Box::new(tx));
            let mut raw: ffi::DNSServiceRef = ptr::null_mut();
            let r = ffi::DNSServiceBrowse(
                &mut raw as _,
                0,
                0,
                service_type.as_ptr(),
                c_domain.map_or(ptr::null_mut(), |d| d.as_ptr()),
                Some(browse_callback),
                tx as _,
            );
            if r != ffi::kDNSServiceErr_NoError {
                error!("DNSServiceBrowser error: {}", r);
                return Err(BrowseError::ServiceError(r));
            }
            let (final_tx, final_rx) = sync_channel::<Result<Service>>(10);
            resolver_thread(rx, final_tx);
            Ok(ServiceBrowser { raw, rx: final_rx })
        }
    }
    /// Returns discovered services if any
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Service> {
        // TODO: do non-blocking check before calling?
        if self.has_data(timeout)? {
            trace!("Data on socket, processing before checking channel");
            let r = self.process_result();
            if r != ffi::kDNSServiceErr_NoError {
                return Err(BrowseError::ServiceError(r));
            }
        }

        match self.rx.recv_timeout(timeout) {
            Ok(service_result) => service_result,
            Err(RecvTimeoutError::Timeout) => {
                Err(BrowseError::IoError(IoError::from(ErrorKind::TimedOut)))
            }
            Err(RecvTimeoutError::Disconnected) => Err(BrowseError::IoError(IoError::from(
                ErrorKind::ConnectionReset,
            ))),
        }
    }
}

impl Drop for ServiceBrowser {
    fn drop(&mut self) {
        unsafe {
            ffi::DNSServiceRefDeallocate(self.raw);
        }
    }
}

// should be safe to send across threads, just not shared
unsafe impl Send for ServiceBrowser {}

pub fn browse(builder: ServiceBrowserBuilder) -> Result<ServiceBrowser> {
    Ok(ServiceBrowser::start(builder.regtype, builder.domain)?)
}
macro_rules! mut_void_ptr {
    ($var:expr) => {
        $var as *mut _ as *mut c_void
    };
}
impl DiscoveredService {
    fn resolve(&self) -> Result<Vec<ResolvedService>> {
        let mut sdref: ffi::DNSServiceRef = unsafe { std::mem::zeroed() };
        let regtype =
            CString::new(self.regtype.as_str()).map_err(|_| BrowseError::InvalidString)?;
        let name = CString::new(self.name.as_str()).map_err(|_| BrowseError::InvalidString)?;
        let domain = CString::new(self.domain.as_str()).map_err(|_| BrowseError::InvalidString)?;
        let mut pending_resolution: PendingResolution = Default::default();
        unsafe {
            let r = ffi::DNSServiceResolve(
                &mut sdref,
                0,
                self.interface_index,
                name.as_ptr(),
                regtype.as_ptr(),
                domain.as_ptr(),
                Some(resolve_callback),
                mut_void_ptr!(&mut pending_resolution),
            );
            if r != kDNSServiceErr_NoError {
                return Err(BrowseError::ServiceError(r));
            }
            while pending_resolution.more_coming {
                ffi::DNSServiceProcessResult(sdref);
            }
            ffi::DNSServiceRefDeallocate(sdref);
        }

        Ok(pending_resolution.results)
    }
}

struct PendingResolution {
    more_coming: bool,
    results: Vec<ResolvedService>,
}
impl Default for PendingResolution {
    fn default() -> Self {
        PendingResolution {
            more_coming: true, // default to true, just as a way to say yes for first entry
            results: Vec::with_capacity(1),
        }
    }
}

/// Resolved service information, name, hostname, port, & TXT record if any
#[derive(Debug)]
pub struct ResolvedService {
    /// Full name of service
    pub full_name: String,
    /// Hostname of service, usable with gethostbyname()
    pub hostname: String,
    /// Port service is on
    pub port: u16,
    /// TXT record service has if any
    pub txt_record: Option<HashMap<String, String>>,
    interface_index: u32,
}
impl ToSocketAddrs for ResolvedService {
    type Iter = std::vec::IntoIter<SocketAddr>;
    /// Leverages Rust's ToSocketAddrs to resolve service hostname & port, host needs integrated bonjour support to work
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.hostname.as_str(), self.port).to_socket_addrs()
    }
}

unsafe extern "C" fn resolve_callback(
    _sd_ref: ffi::DNSServiceRef,
    flags: ffi::DNSServiceFlags,
    interface_index: u32,
    error_code: ffi::DNSServiceErrorType,
    full_name: *const c_char,
    host_target: *const c_char,
    port: u16, // network byte order
    txt_len: u16,
    txt_record: *const u8,
    context: *mut c_void,
) {
    let context: &mut PendingResolution = &mut *(context as *mut PendingResolution);
    if error_code != ffi::kDNSServiceErr_NoError {
        error!("Error resolving service: {}", error_code);
        context.more_coming = false;
        return;
    }
    // flag if we have more records coming so we can fetch them before stopping resolution
    context.more_coming = flags & ffi::kDNSServiceFlagsMoreComing as u32 != 0;
    let process = || -> Result<(String, String)> {
        let c_str: &CStr = CStr::from_ptr(full_name);
        let full_name: &str = c_str
            .to_str()
            .map_err(|_| BrowseError::InternalInvalidString)?;
        let c_str: &CStr = CStr::from_ptr(host_target);
        let hostname: &str = c_str
            .to_str()
            .map_err(|_| BrowseError::InternalInvalidString)?;
        Ok((full_name.to_owned(), hostname.to_owned()))
    };
    let txt_record = if txt_len > 0 {
        let data = std::slice::from_raw_parts(txt_record, txt_len as usize);
        match hash_from_txt(data) {
            Ok(hash) if hash.len() > 0 => Some(hash),
            Ok(_hash) => None,
            Err(e) => {
                error!("Failed to get TXT record: {:?}", e);
                None
            }
        }
    } else {
        None
    };
    match process() {
        Ok((full_name, hostname)) => {
            let service = ResolvedService {
                full_name,
                hostname,
                port: u16::from_be(port),
                txt_record,
                interface_index,
            };
            context.results.push(service);
        }
        Err(e) => {
            error!("Error resolving service: {:?}", e);
        }
    }
}

fn hash_from_txt(data: &[u8]) -> Result<HashMap<String, String>> {
    let slice = data;
    let txt_len = slice.len() as u16;
    let txt_bytes = slice.as_ptr() as *const c_void;

    unsafe {
        let total_keys = ffi::TXTRecordGetCount(txt_len, txt_bytes);
        let mut hash: HashMap<String, String> = HashMap::with_capacity(total_keys as _);
        for i in 0..total_keys {
            // index is u16 so we can't go over u16::max_value() but likely will end before that
            let mut key: [c_char; 256] = std::mem::zeroed();
            let mut value = std::mem::zeroed();
            let mut value_len: u8 = 0;
            let err = ffi::TXTRecordGetItemAtIndex(
                txt_len,
                txt_bytes,
                i,
                key.len() as u16,
                key.as_mut_ptr(),
                &mut value_len,
                &mut value,
            );
            if err == ffi::kDNSServiceErr_NoError {
                let c_str: &CStr = CStr::from_ptr(key.as_ptr());
                let key: &str = c_str.to_str().unwrap();
                let data = std::slice::from_raw_parts(value as *mut u8, value_len as _);
                match std::str::from_utf8(data) {
                    Ok(value) if key.len() > 0 && value.len() > 0 => {
                        hash.insert(key.to_owned(), value.to_owned());
                    }
                    Ok(_value) => {
                        trace!("Discarding TXT key with empty key & value");
                    }
                    Err(e) => {
                        error!("Error processing TXT value as UTF-8: {}", e);
                    }
                }
            }
            if err == ffi::kDNSServiceErr_Invalid {
                error!("Error invalid fetching TXT");
                break;
            }
        }
        Ok(hash)
    }
}
