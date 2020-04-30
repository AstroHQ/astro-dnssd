//! Module containing code related to browsing/searching for services

use crate::ffi;
use crate::DNSServiceError;
// use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::mem;
use std::net::{SocketAddr, ToSocketAddrs};
use std::os::raw::c_char;
use std::ptr;

macro_rules! mut_void_ptr {
    ($var:expr) => {
        $var as *mut _ as *mut c_void
    };
}
macro_rules! mut_raw_ptr {
    ($var:expr) => {
        &mut $var as *mut _
    };
}

/// Type of service event from browser, if a service is being added or removed from network
#[derive(Debug)]
pub enum ServiceEventType {
    /// Service has been added to the network
    Added,
    /// Service is removed from the network
    Removed,
}
impl From<ffi::DNSServiceFlags> for ServiceEventType {
    fn from(flags: ffi::DNSServiceFlags) -> Self {
        if flags & ffi::kDNSServiceFlagsAdd as u32 != 0 {
            ServiceEventType::Added
        } else {
            ServiceEventType::Removed
        }
    }
}
/// Encapsulates information about a service
#[derive(Debug)]
pub struct Service {
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
    pub txt_record: Option<TXTHash>,
    interface_index: u32,
}
impl ToSocketAddrs for ResolvedService {
    type Iter = std::vec::IntoIter<SocketAddr>;
    /// Leverages Rust's ToSocketAddrs to resolve service hostname & port, host needs integrated bonjour support to work
    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        (self.hostname.as_str(), self.port).to_socket_addrs()
    }
}

/// Defines which IP type to resolve a host for
pub enum ResolveIpType {
    /// Resolve IPv4
    V4,
    /// Resolve IPv6
    V6,
}
impl Into<ffi::DNSServiceProtocol> for ResolveIpType {
    fn into(self) -> ffi::DNSServiceProtocol {
        match self {
            ResolveIpType::V4 => ffi::kDNSServiceProtocol_IPv4 as u32,
            ResolveIpType::V6 => ffi::kDNSServiceProtocol_IPv6 as u32,
        }
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

impl Service {
    /// Resolves service to get it's hostname, port, etc. Blocks until response is received
    pub fn resolve(&mut self) -> Result<Vec<ResolvedService>, DNSServiceError> {
        let mut sdref: ffi::DNSServiceRef = unsafe { mem::zeroed() };
        let regtype =
            CString::new(self.regtype.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
        let name = CString::new(self.name.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
        let domain =
            CString::new(self.domain.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
        let mut pending_resolution: PendingResolution = Default::default();
        unsafe {
            ffi::DNSServiceResolve(
                mut_raw_ptr!(sdref),
                0,
                self.interface_index,
                name.as_ptr(),
                regtype.as_ptr(),
                domain.as_ptr(),
                Some(Self::resolve_callback),
                mut_void_ptr!(&mut pending_resolution),
            );
            while pending_resolution.more_coming {
                ffi::DNSServiceProcessResult(sdref);
            }
            ffi::DNSServiceRefDeallocate(sdref);
        }

        Ok(pending_resolution.results)
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
        let process = || -> Result<(String, String), DNSServiceError> {
            let c_str: &CStr = CStr::from_ptr(full_name);
            let full_name: &str = c_str
                .to_str()
                .map_err(|_| DNSServiceError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(host_target);
            let hostname: &str = c_str
                .to_str()
                .map_err(|_| DNSServiceError::InternalInvalidString)?;
            Ok((full_name.to_owned(), hostname.to_owned()))
        };
        let txt_record = if txt_len > 0 {
            let data = std::slice::from_raw_parts(txt_record, txt_len as usize).to_vec();
            match TXTHash::new(data) {
                Ok(hash) => {
                    trace!("Got TXT: {:?}", hash);
                    Some(hash)
                }
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
                trace!(
                    "{} - {} service resolved",
                    service.full_name,
                    service.hostname
                );
                context.results.push(service);
            }
            Err(e) => {
                error!("Error resolving service: {:?}", e);
            }
        }
    }
}

// TODO: figure out a nicer TXT record API that handles the difference better between creation & reading
/// Read only owned TXTRecord returned by service resolution & querying
#[derive(Debug)]
pub struct TXTHash {
    data: Vec<u8>,
}
impl TXTHash {
    /// Creates new hash from bytes
    pub fn new(data: Vec<u8>) -> Result<Self, DNSServiceError> {
        Ok(TXTHash { data })
    }
    fn as_raw(&self) -> (u16, *const c_void) {
        (self.data.len() as u16, self.data.as_ptr() as *const c_void)
    }
    /// Returns true if the given key has an entry in the TXTRecord
    pub fn contains(&self, key: &str) -> bool {
        let key_c = CString::new(key).unwrap();
        unsafe {
            let (txt_len, txt_data) = self.as_raw();
            if ffi::TXTRecordContainsKey(txt_len, txt_data, key_c.as_ptr()) == 0 {
                return false;
            }
        }
        true
    }
    /// Returns value for given key if it exists
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let key_c = CString::new(key).unwrap();

        let mut value_len: u8 = 0;
        let (txt_len, txt_data) = self.as_raw();
        unsafe {
            if !self.contains(key) {
                return None;
            }
            let data_ptr =
                ffi::TXTRecordGetValuePtr(txt_len, txt_data, key_c.as_ptr(), &mut value_len);
            let slice = std::slice::from_raw_parts(data_ptr as *const u8, value_len as usize);
            Some(slice.to_vec())
        }
    }
}
// impl Into<HashMap<String, Vec<u8>>> for TXTHash {
//     fn into(self) -> HashMap<String, Vec<u8>> {
//         let map = HashMap::new();
//         let slice = &self.data[..];
//         // if slice.len() > u16::max_value() as usize {
//         //     error!(
//         //         "TXTHash bytes data too large, {} larger than u16 limit",
//         //         slice.len()
//         //     );
//         //     return Err(DNSServiceError::InvalidString);
//         // }
//         let txt_len = slice.len() as u16;
//         let txt_bytes = slice.as_ptr() as *const c_void;
//         let mut hash: HashMap<String, Vec<u8>> = HashMap::new();
//         unsafe {
//             let total_keys = ffi::TXTRecordGetCount(txt_len, txt_bytes);
//             for i in 0..total_keys {
//                 // index is u16 so we can't go over u16::max_value() but likely will end before that
//                 let mut key: [c_char; 256] = mem::zeroed();
//                 let mut value: [u8; u8::max_value() as usize] = mem::zeroed();
//                 let mut value_len: u8 = 0;
//                 let err = ffi::TXTRecordGetItemAtIndex(
//                     txt_len,
//                     txt_bytes,
//                     i,
//                     key.len() as u16,
//                     key.as_mut_ptr(),
//                     &mut value_len,
//                     value.as_mut_ptr() as *mut *const c_void,
//                 );
//                 trace!("Got value len: {}", value_len);
//                 if err == ffi::kDNSServiceErr_NoError {
//                     let c_str: &CStr = CStr::from_ptr(key.as_ptr());
//                     let key: &str = c_str.to_str().unwrap();
//                     // TODO: figure out proper way to do this with a slice etc
//                     let data = (&value[0..value_len as usize]).to_vec();
//                     hash.insert(key.to_owned(), data);
//                 }
//                 if err == ffi::kDNSServiceErr_Invalid {
//                     break;
//                 }
//             }
//         }
//         map
//     }
// }

/// Builder for creating a browser, allowing optionally specifying a domain with chaining (maybe builder is excessive)
pub struct ServiceBrowserBuilder {
    regtype: String,
    domain: Option<String>,
}

impl ServiceBrowserBuilder {
    /// Creates new service browser for given service type, i.e. ._http._tcp.
    pub fn new(regtype: &str) -> ServiceBrowserBuilder {
        ServiceBrowserBuilder {
            regtype: String::from(regtype),
            domain: None,
        }
    }
    /// Adds a specified domain to browser's search
    pub fn with_domain(mut self, domain: &str) -> ServiceBrowserBuilder {
        self.domain = Some(String::from(domain));
        self
    }

    /// Creates browser and starts searching,
    pub fn build(self) -> Result<DNSServiceBrowser, DNSServiceError> {
        unsafe {
            let service = DNSServiceBrowser {
                regtype: self.regtype,
                domain: self.domain,
                raw: mem::zeroed(),
                // TODO: replace this? think it might live forever
                reply_callback: Box::new(|_| {}),
            };
            Ok(service)
        }
    }
}

/// Main service browser, calls callback upon discovery of service
pub struct DNSServiceBrowser {
    /// Type to search for, i.e. ._http._tcp.
    pub regtype: String,
    /// Domain to search in, default is .local
    pub domain: Option<String>,
    raw: ffi::DNSServiceRef,
    reply_callback: Box<dyn Fn(Result<Service, DNSServiceError>) -> ()>,
}

impl DNSServiceBrowser {
    unsafe extern "C" fn reply_callback(
        _sd_ref: ffi::DNSServiceRef,
        flags: ffi::DNSServiceFlags,
        interface_index: u32,
        error_code: ffi::DNSServiceErrorType,
        service_name: *const c_char,
        regtype: *const c_char,
        reply_domain: *const c_char,
        context: *mut c_void,
    ) {
        let context: &mut DNSServiceBrowser = &mut *(context as *mut DNSServiceBrowser);

        // shouldn't need any other args if there's an error
        if error_code != 0 {
            (context.reply_callback)(Err(DNSServiceError::ServiceError(error_code)));
            return;
        }

        // build Strings from c_char
        let process = || -> Result<(String, String, String), DNSServiceError> {
            let c_str: &CStr = CStr::from_ptr(service_name);
            let service_name: &str = c_str
                .to_str()
                .map_err(|_| DNSServiceError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(regtype);
            let regtype: &str = c_str
                .to_str()
                .map_err(|_| DNSServiceError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(reply_domain);
            let reply_domain: &str = c_str
                .to_str()
                .map_err(|_| DNSServiceError::InternalInvalidString)?;
            Ok((
                service_name.to_owned(),
                regtype.to_owned(),
                reply_domain.to_owned(),
            ))
        };
        match process() {
            Ok((name, regtype, domain)) => {
                let service = Service {
                    name,
                    regtype,
                    interface_index,
                    domain,
                    event_type: flags.into(),
                };
                (context.reply_callback)(Ok(service));
            }
            Err(e) => {
                (context.reply_callback)(Err(e));
            }
        }
    }

    /// Returns socket to mDNS service, use with select()
    pub fn socket(&self) -> i32 {
        unsafe { ffi::DNSServiceRefSockFD(self.raw) }
    }

    /// Processes a reply from mDNS service, blocking until there is one
    pub fn process_result(&self) -> ffi::DNSServiceErrorType {
        unsafe { ffi::DNSServiceProcessResult(self.raw) }
    }

    //     /// returns true if the socket has data and process_result() should be called
    // pub fn has_data(&self) -> bool {
    //     unsafe {
    //         let fd = self.socket();
    //         let mut timeout = libc::timeval { tv_sec: 5, tv_usec: 0 };
    //         let mut read_set = mem::uninitialized();
    //         libc::FD_ZERO(&mut read_set);
    //         libc::FD_SET(fd, &mut read_set);
    //         libc::select(fd + 1, &mut read_set, ptr::null_mut(), ptr::null_mut(), &mut timeout);
    //         libc::FD_ISSET(fd, &mut read_set)
    //     }
    // }

    /// Starts browser with given callback that'll be called upon discovery
    pub fn start<F: 'static>(&mut self, callback: F) -> Result<(), DNSServiceError>
    where
        F: Fn(Result<Service, DNSServiceError>) -> (),
    {
        // TODO: figure out if we can have non-'static callback
        self.reply_callback = Box::new(callback);
        unsafe {
            let c_domain: Option<CString>;
            if let Some(d) = &self.domain {
                c_domain =
                    Some(CString::new(d.as_str()).map_err(|_| DNSServiceError::InvalidString)?);
            } else {
                c_domain = None;
            }
            let service_type =
                CString::new(self.regtype.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
            ffi::DNSServiceBrowse(
                &mut self.raw as *mut _,
                0,
                0,
                service_type.as_ptr(),
                c_domain.map_or(ptr::null_mut(), |d| d.as_ptr()),
                Some(DNSServiceBrowser::reply_callback),
                mut_void_ptr!(self),
            );
            Ok(())
        }
    }
}

impl Drop for DNSServiceBrowser {
    fn drop(&mut self) {
        unsafe {
            ffi::DNSServiceRefDeallocate(self.raw);
        }
    }
}
