use crate::ffi::apple as ffi;
// use std::collections::HashMap;
use crate::browse::{Service, ServiceEventType};
use crate::ffi::apple::kDNSServiceErr_NoError;
use crate::ServiceBrowserBuilder;
use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::io::{Error as IoError, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs};
use std::os::raw::c_char;
use std::ptr;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::{Duration, Instant};
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
    /// Timeout error when waiting for more data from browser
    #[error("Timeout waiting for more data")]
    Timeout,
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
                let service = DiscoveredService {
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
    let (port, hostname, txt_record, resolved_interface) = match resolved.into_iter().next() {
        Some(resolved) => (
            resolved.port,
            resolved.hostname,
            resolved.txt_record,
            resolved.interface_index,
        ),
        None => (0, "".to_string(), None, 0),
    };
    // Browse can report a placeholder like kDNSServiceInterfaceIndexP2P, resolve reports the concrete interface
    let interface_index = if resolved_interface != 0 {
        resolved_interface
    } else {
        discovered.interface_index
    };
    Service {
        name: discovered.name,
        domain: discovered.domain,
        regtype: discovered.regtype,
        interface_index: Some(interface_index),
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
                            if let Err(_e) = tx.send(Ok(service)) {
                                error!("Error sending resolved service, disconnected channel, exiting thread");
                                break;
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
                    warn!("Resolver channel disconnected, exiting thread as we're likely stopped/dropped");
                    break;
                }
            }
        }).expect("Failed to start resolver thread");
}

/// Main service browser, calls callback upon discovery of service
pub struct ServiceBrowser {
    /// Raw DNS-SD service reference
    raw: ffi::DNSServiceRef,
    /// Receiver to receive successfully discovered & resolved services from
    rx: Receiver<Result<Service>>,
    /// Raw pointer the browse callback uses for the SyncSender, to use with Box::from_raw() during Drop
    raw_tx: *mut SyncSender<Result<DiscoveredService>>,
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

    /// Starts browser with type, optional domain & flags
    fn start(regtype: String, domain: Option<String>, flags: ffi::DNSServiceFlags) -> Result<Self> {
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
                flags,
                0,
                service_type.as_ptr(),
                c_domain.map_or(ptr::null_mut(), |d| d.as_ptr()),
                Some(browse_callback),
                tx as _,
            );
            if r != kDNSServiceErr_NoError {
                error!("DNSServiceBrowser error: {}", r);
                return Err(BrowseError::ServiceError(r));
            }
            let (final_tx, final_rx) = sync_channel::<Result<Service>>(10);
            resolver_thread(rx, final_tx);
            Ok(ServiceBrowser {
                raw,
                rx: final_rx,
                raw_tx: tx,
            })
        }
    }
    /// Returns discovered services if any
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Service> {
        // TODO: do non-blocking check before calling?
        if self.has_data(timeout)? {
            trace!("Data on socket, processing before checking channel");
            let r = self.process_result();
            if r != kDNSServiceErr_NoError {
                return Err(BrowseError::ServiceError(r));
            }
        }

        match self.rx.recv_timeout(timeout) {
            Ok(service_result) => service_result,
            Err(RecvTimeoutError::Timeout) => Err(BrowseError::Timeout),
            Err(RecvTimeoutError::Disconnected) => Err(BrowseError::IoError(IoError::from(
                ErrorKind::ConnectionReset,
            ))),
        }
    }
}

impl Drop for ServiceBrowser {
    fn drop(&mut self) {
        unsafe {
            // ensure we cancel browser first by deallocating it...
            ffi::DNSServiceRefDeallocate(self.raw);
            // then we should be able to safely drop the sender which will signal resolver thread to exit
            let _tx = Box::from_raw(self.raw_tx);
        }
    }
}

// should be safe to send across threads, just not shared
unsafe impl Send for ServiceBrowser {}

pub fn browse(builder: ServiceBrowserBuilder) -> Result<ServiceBrowser> {
    #[cfg(target_vendor = "apple")]
    let flags = super::peer_to_peer_flags(builder.peer_to_peer);
    #[cfg(not(target_vendor = "apple"))]
    let flags = 0;
    ServiceBrowser::start(builder.regtype, builder.domain, flags)
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
            #[allow(clippy::while_immutable_condition)]
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
    #[allow(dead_code)] // TODO: check if we should expose this to public API?
    pub full_name: String,
    /// Hostname of service, usable with gethostbyname()
    pub hostname: String,
    /// Port service is on
    pub port: u16,
    /// TXT record service has if any
    pub txt_record: Option<HashMap<String, String>>,
    /// Interface the service resolved on, concrete even when browse reported kDNSServiceInterfaceIndexP2P
    pub interface_index: u32,
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
    if error_code != kDNSServiceErr_NoError {
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
            Ok(hash) if !hash.is_empty() => Some(hash),
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
        // SAFETY: generated txt_bytes from rust slice with guarantees
        let total_keys = ffi::TXTRecordGetCount(txt_len, txt_bytes);
        let mut hash: HashMap<String, String> = HashMap::with_capacity(total_keys as _);
        for i in 0..total_keys {
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
            if err == kDNSServiceErr_NoError {
                let c_str: &CStr = CStr::from_ptr(key.as_ptr());
                let key: &str = c_str.to_str().unwrap();
                // From the API documentation
                // For keys with no value, *value is set to NULL and *valueLen is zero.
                if value.is_null() {
                    if !key.is_empty() {
                        // use html notion of key==value for key only term to be able to represent
                        // it in the hashmap without having to change the API
                        hash.insert(key.to_owned(), key.to_owned());
                    } else {
                        trace!("Discarding TXT key with empty key & null value");
                    }
                } else {
                    // SAFETY: guarding against null value above, otherwise trusting the C API
                    let data = std::slice::from_raw_parts(value as *mut u8, value_len as _);
                    match std::str::from_utf8(data) {
                        Ok(value) if !key.is_empty() && !value.is_empty() => {
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
            }
            if err == ffi::kDNSServiceErr_Invalid {
                error!("Error invalid fetching TXT");
                break;
            }
        }
        Ok(hash)
    }
}

/// kDNSServiceInterfaceIndexP2P, bindgen skips it as it's a cast in a `#define`. Only valid for browse, query record,
/// register & resolve, not address lookups
#[cfg(target_vendor = "apple")]
const INTERFACE_INDEX_P2P: u32 = 0xffff_fffd;

/// Longest single wait on the mDNSResponder socket while looking up addresses, keeps `select`'s timeout under a second
#[cfg(target_vendor = "apple")]
const ADDR_INFO_POLL: Duration = Duration::from_millis(100);

/// Addresses found so far for one [`resolve_socket_addrs`] lookup, filled in by [`addr_info_callback`]
#[cfg(target_vendor = "apple")]
struct PendingAddrInfo {
    port: u16,
    interface_index: u32,
    more_coming: bool,
    error: Option<ffi::DNSServiceErrorType>,
    addrs: Vec<SocketAddr>,
    /// mDNSResponder answered that the host has no IPv4 address
    no_ipv4: bool,
    /// mDNSResponder answered that the host has no IPv6 address
    no_ipv6: bool,
}

/// See [`crate::Service::resolve_socket_addrs`]. `interface_index` 0 looks on every interface, peer-to-peer ones too
#[cfg(target_vendor = "apple")]
pub fn resolve_socket_addrs(
    hostname: &str,
    port: u16,
    interface_index: u32,
    timeout: Duration,
) -> Result<Vec<SocketAddr>> {
    let c_hostname = CString::new(hostname).map_err(|_| BrowseError::InvalidString)?;
    // Only left over if resolve didn't report a concrete interface, look everywhere (P2P included) instead
    let interface_index = if interface_index == INTERFACE_INDEX_P2P {
        ffi::kDNSServiceInterfaceIndexAny
    } else {
        interface_index
    };
    let lookup = Box::into_raw(Box::new(PendingAddrInfo {
        port,
        interface_index,
        more_coming: true,
        error: None,
        addrs: Vec::new(),
        no_ipv4: false,
        no_ipv6: false,
    }));
    let mut sdref: ffi::DNSServiceRef = ptr::null_mut();
    let r = unsafe {
        ffi::DNSServiceGetAddrInfo(
            &mut sdref,
            super::peer_to_peer_flags(true),
            interface_index,
            (ffi::kDNSServiceProtocol_IPv4 | ffi::kDNSServiceProtocol_IPv6) as _,
            c_hostname.as_ptr(),
            Some(addr_info_callback),
            lookup as *mut c_void,
        )
    };
    if r != kDNSServiceErr_NoError {
        // SAFETY: mDNSResponder didn't take the context, so it's still only ours
        drop(unsafe { Box::from_raw(lookup) });
        return Err(BrowseError::ServiceError(r));
    }

    let deadline = Instant::now() + timeout;
    let result = loop {
        // SAFETY: the callback only runs inside DNSServiceProcessResult below, never while we read
        let pending = unsafe { &*lookup };
        if let Some(e) = pending.error {
            break Err(BrowseError::ServiceError(e));
        }
        if !pending.more_coming {
            if !pending.addrs.is_empty() {
                break Ok(());
            }
            if pending.no_ipv4 && pending.no_ipv6 {
                break Err(BrowseError::ServiceError(ffi::kDNSServiceErr_NoSuchRecord));
            }
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break if pending.addrs.is_empty() {
                Err(BrowseError::Timeout)
            } else {
                Ok(())
            };
        }
        let fd = unsafe { ffi::DNSServiceRefSockFD(sdref) };
        match crate::non_blocking::socket_is_ready(fd as _, remaining.min(ADDR_INFO_POLL)) {
            Ok(true) => {
                let r = unsafe { ffi::DNSServiceProcessResult(sdref) };
                if r != kDNSServiceErr_NoError {
                    break Err(BrowseError::ServiceError(r));
                }
            }
            Ok(false) => {}
            Err(e) => break Err(e.into()),
        }
    };

    // SAFETY: deallocating first stops any more callbacks, then the context is only ours again
    unsafe { ffi::DNSServiceRefDeallocate(sdref) };
    let pending = unsafe { Box::from_raw(lookup) };
    result.map(|()| pending.addrs)
}

#[cfg(target_vendor = "apple")]
unsafe extern "C" fn addr_info_callback(
    _sd_ref: ffi::DNSServiceRef,
    flags: ffi::DNSServiceFlags,
    interface_index: u32,
    error_code: ffi::DNSServiceErrorType,
    _hostname: *const c_char,
    address: *const ffi::sockaddr,
    _ttl: u32,
    context: *mut c_void,
) {
    let pending: &mut PendingAddrInfo = &mut *(context as *mut PendingAddrInfo);
    if error_code == ffi::kDNSServiceErr_NoSuchRecord {
        // Negative answer for one address family (address is a zeroed one of that family), the other can still
        // have addresses, i.e. IPv6-only AWDL peers
        pending.more_coming = flags & ffi::kDNSServiceFlagsMoreComing != 0;
        if !address.is_null() {
            match (*(address as *const libc::sockaddr)).sa_family as i32 {
                libc::AF_INET => pending.no_ipv4 = true,
                libc::AF_INET6 => pending.no_ipv6 = true,
                _ => {}
            }
        }
        return;
    }
    if error_code != kDNSServiceErr_NoError {
        error!("Error looking up addresses: {}", error_code);
        pending.error = Some(error_code);
        return;
    }
    pending.more_coming = flags & ffi::kDNSServiceFlagsMoreComing != 0;
    if flags & ffi::kDNSServiceFlagsAdd == 0 || address.is_null() {
        return;
    }
    let interface_index = if interface_index != 0 {
        interface_index
    } else {
        pending.interface_index
    };
    if let Some(addr) = socket_addr_from_raw(
        address as *const libc::sockaddr,
        pending.port,
        interface_index,
    ) {
        if !pending.addrs.contains(&addr) {
            pending.addrs.push(addr);
        }
    }
}

/// Converts a C socket address to a Rust one with `port`. A link-local IPv6 address without a scope gets
/// `interface_index`, it can't be used without one
#[cfg(target_vendor = "apple")]
unsafe fn socket_addr_from_raw(
    address: *const libc::sockaddr,
    port: u16,
    interface_index: u32,
) -> Option<SocketAddr> {
    match (*address).sa_family as i32 {
        libc::AF_INET => {
            let v4 = &*(address as *const libc::sockaddr_in);
            let ip = Ipv4Addr::from(u32::from_be(v4.sin_addr.s_addr));
            Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        }
        libc::AF_INET6 => {
            let v6 = &*(address as *const libc::sockaddr_in6);
            let ip = Ipv6Addr::from(v6.sin6_addr.s6_addr);
            let link_local = ip.segments()[0] & 0xffc0 == 0xfe80;
            let scope = if v6.sin6_scope_id == 0 && link_local {
                interface_index
            } else {
                v6.sin6_scope_id
            };
            Some(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, scope)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn discovered(interface_index: u32) -> DiscoveredService {
        DiscoveredService {
            name: "name".into(),
            regtype: "_test._tcp".into(),
            interface_index,
            domain: "local.".into(),
            event_type: ServiceEventType::Added,
        }
    }

    fn resolved(interface_index: u32) -> ResolvedService {
        ResolvedService {
            full_name: "name._test._tcp.local.".into(),
            hostname: "host.local.".into(),
            port: 1234,
            txt_record: None,
            interface_index,
        }
    }

    fn pending_addr_info() -> PendingAddrInfo {
        PendingAddrInfo {
            port: 1234,
            interface_index: 19,
            more_coming: true,
            error: None,
            addrs: Vec::new(),
            no_ipv4: false,
            no_ipv6: false,
        }
    }

    fn sockaddr_v4(ip: Ipv4Addr) -> libc::sockaddr_in {
        let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
        addr.sin_len = std::mem::size_of::<libc::sockaddr_in>() as _;
        addr.sin_family = libc::AF_INET as _;
        addr.sin_addr.s_addr = u32::from(ip).to_be();
        addr
    }

    fn sockaddr_v6(ip: Ipv6Addr) -> libc::sockaddr_in6 {
        let mut addr: libc::sockaddr_in6 = unsafe { std::mem::zeroed() };
        addr.sin6_len = std::mem::size_of::<libc::sockaddr_in6>() as _;
        addr.sin6_family = libc::AF_INET6 as _;
        addr.sin6_addr.s6_addr = ip.octets();
        addr
    }

    /// Feeds one GetAddrInfo answer to the callback like mDNSResponder would
    fn answer<T>(
        pending: &mut PendingAddrInfo,
        error_code: ffi::DNSServiceErrorType,
        more_coming: bool,
        address: &T,
    ) {
        let mut flags = ffi::kDNSServiceFlagsAdd;
        if more_coming {
            flags |= ffi::kDNSServiceFlagsMoreComing;
        }
        unsafe {
            addr_info_callback(
                ptr::null_mut(),
                flags,
                19,
                error_code,
                ptr::null(),
                address as *const T as *const ffi::sockaddr,
                120,
                pending as *mut PendingAddrInfo as *mut c_void,
            )
        }
    }

    #[test]
    fn missing_ipv4_keeps_ipv6_addresses() {
        let mut pending = pending_addr_info();
        answer(
            &mut pending,
            ffi::kDNSServiceErr_NoSuchRecord,
            true,
            &sockaddr_v4(Ipv4Addr::UNSPECIFIED),
        );
        let ip: Ipv6Addr = "fe80::1".parse().unwrap();
        answer(
            &mut pending,
            kDNSServiceErr_NoError,
            false,
            &sockaddr_v6(ip),
        );
        assert_eq!(pending.error, None);
        assert!(pending.no_ipv4 && !pending.no_ipv6);
        assert!(!pending.more_coming);
        assert_eq!(
            pending.addrs,
            vec![SocketAddr::V6(SocketAddrV6::new(ip, 1234, 0, 19))]
        );
    }

    #[test]
    fn missing_ipv6_after_ipv4_address_is_not_an_error() {
        let mut pending = pending_addr_info();
        let ip = Ipv4Addr::new(192, 168, 1, 2);
        answer(&mut pending, kDNSServiceErr_NoError, true, &sockaddr_v4(ip));
        answer(
            &mut pending,
            ffi::kDNSServiceErr_NoSuchRecord,
            false,
            &sockaddr_v6(Ipv6Addr::UNSPECIFIED),
        );
        assert_eq!(pending.error, None);
        assert!(pending.no_ipv6);
        assert_eq!(
            pending.addrs,
            vec![SocketAddr::V4(SocketAddrV4::new(ip, 1234))]
        );
    }

    #[test]
    fn other_errors_still_fail_lookup() {
        let mut pending = pending_addr_info();
        answer(
            &mut pending,
            ffi::kDNSServiceErr_Unknown,
            false,
            &sockaddr_v4(Ipv4Addr::UNSPECIFIED),
        );
        assert_eq!(pending.error, Some(ffi::kDNSServiceErr_Unknown));
    }

    #[test]
    fn resolved_interface_replaces_p2p_placeholder() {
        let service = service_from_resolved(discovered(0xffff_fffd), vec![resolved(19)]);
        assert_eq!(service.interface_index, Some(19));
    }

    #[test]
    fn browse_interface_kept_without_resolved_interface() {
        let service = service_from_resolved(discovered(7), vec![resolved(0)]);
        assert_eq!(service.interface_index, Some(7));
        let service = service_from_resolved(discovered(7), vec![]);
        assert_eq!(service.interface_index, Some(7));
    }
}
