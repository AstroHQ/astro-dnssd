use crate::ffi::apple as ffi;
// use std::collections::HashMap;
use crate::browse::{Service, ServiceEventType};
use crate::ServiceBrowserBuilder;
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
    if !context.is_null() {
        let tx_ptr: *mut SyncSender<Result<Service>> = context as _;
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
                let service = Service {
                    name,
                    regtype,
                    interface_index,
                    domain,
                    event_type: flags.into(),
                };
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
    pub fn has_data(&self, timeout: Duration) -> Result<bool> {
        let socket = unsafe { ffi::DNSServiceRefSockFD(self.raw) } as _;
        let r = crate::non_blocking::socket_is_ready(socket, timeout)?;
        Ok(r)
        // unsafe {
        //     let fd = self.socket();
        //     let mut timeout = libc::timeval { tv_sec: 5, tv_usec: 0 };
        //     let mut read_set = mem::uninitialized();
        //     libc::FD_ZERO(&mut read_set);
        //     libc::FD_SET(fd, &mut read_set);
        //     libc::select(fd + 1, &mut read_set, ptr::null_mut(), ptr::null_mut(), &mut timeout);
        //     libc::FD_ISSET(fd, &mut read_set)
        // }
    }

    /// Starts browser with given callback that'll be called upon discovery
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
            let (tx, rx) = sync_channel::<Result<Service>>(10);
            let tx = Box::into_raw(Box::new(tx));
            let mut raw: ffi::DNSServiceRef = ptr::null_mut();
            let r = ffi::DNSServiceBrowse(
                &mut raw as _,
                0,
                0,
                service_type.as_ptr(),
                c_domain.map_or(ptr::null_mut(), |d| d.as_ptr()),
                Some(reply_callback),
                tx as _,
            );
            if r != ffi::kDNSServiceErr_NoError {
                error!("DNSServiceBrowser error: {}", r);
                return Err(BrowseError::ServiceError(r));
            }
            Ok(ServiceBrowser { raw, rx })
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
