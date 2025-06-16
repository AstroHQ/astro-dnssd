//! Registration of dns-sd services

// use super::txt::TXTRecord;
use crate::{
    ffi::apple::{
        kDNSServiceErr_NoError, kDNSServiceErr_ServiceNotRunning, DNSServiceErrorType,
        DNSServiceFlags, DNSServiceProcessResult, DNSServiceRef, DNSServiceRefDeallocate,
        DNSServiceRefSockFD, DNSServiceRegister,
    },
    os::apple::txt::TXTRecord,
    register::Result,
    DNSServiceBuilder,
};
use std::{
    ffi::{c_void, CStr, CString},
    fmt,
    os::raw::c_char,
    ptr::{null, null_mut},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, SyncSender},
        Arc,
    },
    time::Duration,
};
use thiserror::Error;

const CALLBACK_TIMEOUT: Duration = Duration::from_secs(10);

/// Common error for DNS-SD service
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum RegistrationError {
    /// Invalid input string
    #[error("Invalid string argument, must be C string compatible")]
    InvalidString,
    /// Unexpected invalid strings from C API
    #[error("DNSSD API returned invalid UTF-8 string")]
    InternalInvalidString,
    /// Error from DNSSD service
    #[error("DNSSD Error: {0}")]
    ServiceError(i32),
}

unsafe extern "C" fn register_reply(
    _sd_ref: DNSServiceRef,
    _flags: DNSServiceFlags,
    error_code: DNSServiceErrorType,
    name: *const c_char,
    regtype: *const c_char,
    domain: *const c_char,
    context: *mut c_void,
) {
    info!("Got reply");
    // let context: &mut RegisteredDnsService = &mut *(context as *mut RegisteredDnsService);
    let process = || -> Result<(String, String, String)> {
        let c_str: &CStr = CStr::from_ptr(name);
        let service_name: &str = c_str
            .to_str()
            .map_err(|_| RegistrationError::InternalInvalidString)?;
        let c_str: &CStr = CStr::from_ptr(regtype);
        let regtype: &str = c_str
            .to_str()
            .map_err(|_| RegistrationError::InternalInvalidString)?;
        let c_str: &CStr = CStr::from_ptr(domain);
        let reply_domain: &str = c_str
            .to_str()
            .map_err(|_| RegistrationError::InternalInvalidString)?;
        Ok((
            service_name.to_owned(),
            regtype.to_owned(),
            reply_domain.to_owned(),
        ))
    };
    if !context.is_null() {
        let tx_ptr: *mut SyncSender<Result<DNSServiceRegisterReply>> = context as _;
        let tx = &*tx_ptr;
        trace!("Registration replied");
        match process() {
            Ok((name, regtype, domain)) => {
                if error_code == kDNSServiceErr_NoError {
                    let reply = DNSServiceRegisterReply {
                        regtype,
                        name,
                        domain,
                    };
                    tx.send(Ok(reply)).unwrap();
                    info!("Reply info sent");
                } else {
                    error!("Error in reply: {}", error_code);
                    tx.send(Err(RegistrationError::ServiceError(error_code)))
                        .unwrap();
                }
            }
            Err(e) => {
                error!("Error in reply: {:?}", e);
                tx.send(Err(e)).unwrap();
            }
        }
    }
}

/// DNS-SD Service for registration use
pub struct RegisteredDnsService {
    shutdown_flag: Arc<AtomicBool>,
}
impl fmt::Debug for RegisteredDnsService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegisteredDnsService")
    }
}

/// Reply information upon successful registration
#[derive(Debug)]
#[allow(dead_code)] // TODO: in future look to see if we should use these values?
pub struct DNSServiceRegisterReply {
    /// Service type of successfully registered service
    pub regtype: String,
    /// Name of service
    pub name: String,
    /// Domain used for successful registration
    pub domain: String,
}

/// Service ref to encapsulate DNSServiceRef to send to a thread & cleanup on drop
struct ServiceRef {
    raw: DNSServiceRef,
    context: *mut c_void,
}
impl ServiceRef {
    fn new(raw: DNSServiceRef, context: *mut c_void) -> Self {
        ServiceRef { raw, context }
    }
}
unsafe impl Send for ServiceRef {}
impl Drop for ServiceRef {
    fn drop(&mut self) {
        unsafe {
            trace!("Dropping service");
            if !self.raw.is_null() {
                trace!("Deallocating DNSServiceRef");
                DNSServiceRefDeallocate(self.raw);
                self.raw = null_mut();
                _ = Box::from_raw(self.context);
            }
        }
    }
}

impl RegisteredDnsService {}

// Signal the background thread via an atomic flag
impl Drop for RegisteredDnsService {
    fn drop(&mut self) {
        trace!("Signaling thread to exit...");
        self.shutdown_flag.store(true, Ordering::Release);
    }
}
fn run_thread_with_poll(service: ServiceRef, shutdown_flag: Arc<AtomicBool>) {
    let socket = unsafe { DNSServiceRefSockFD(service.raw) };

    std::thread::spawn(move || {
        loop {
            // Check a shutdown flag
            if shutdown_flag.load(Ordering::Acquire) {
                trace!("Shutdown requested, exiting DNS-SD processing thread");
                break;
            }

            unsafe {
                let mut poll_fd = libc::pollfd {
                    fd: socket,
                    events: libc::POLLIN,
                    revents: 0,
                };

                // 100 ms timeout
                let result = libc::poll(&mut poll_fd, 1, 100);

                match result {
                    -1 => {
                        let errno = *libc::__error();
                        if errno == libc::EINTR {
                            continue;
                        } else if errno == libc::EBADF {
                            trace!("DNS-SD socket closed, exiting thread");
                            break;
                        } else {
                            error!("poll() error: {}, exiting thread", errno);
                            break;
                        }
                    }
                    0 => {
                        // Timeout - continue to check the shutdown flag
                        continue;
                    }
                    _ => {
                        // Check what events occurred
                        if poll_fd.revents & libc::POLLIN != 0 {
                            // Data ready to read
                            trace!("Processing DNS-SD result...");
                            let r = DNSServiceProcessResult(service.raw);
                            if r != kDNSServiceErr_NoError {
                                if r == kDNSServiceErr_ServiceNotRunning {
                                    trace!("DNS-SD service stopped, exiting thread");
                                } else {
                                    error!("Error processing DNS-SD result: {}, exiting thread", r);
                                }
                                break;
                            }
                        } else if poll_fd.revents & (libc::POLLHUP | libc::POLLERR | libc::POLLNVAL)
                            != 0
                        {
                            // Socket error or hangup
                            trace!(
                                "DNS-SD socket error/hangup (revents: {}), exiting thread",
                                poll_fd.revents
                            );
                            break;
                        }
                    }
                }
            }
        }

        trace!("DNS-SD processing thread exited");
    });
}

pub fn register_service(service: DNSServiceBuilder) -> Result<RegisteredDnsService> {
    unsafe {
        let c_name: Option<CString>;
        if let Some(n) = &service.name {
            c_name = Some(CString::new(n.as_str()).map_err(|_| RegistrationError::InvalidString)?);
        } else {
            c_name = None;
        }
        let c_name = c_name.as_ref();
        let service_type =
            CString::new(service.regtype.as_str()).map_err(|_| RegistrationError::InvalidString)?;
        let txt = service.txt.map(TXTRecord::from);
        let (txt_record, txt_len) = match &txt {
            Some(txt) => (txt.raw_bytes_ptr(), txt.raw_bytes_len()),
            None => (null(), 0),
        };

        let (tx, rx) = sync_channel::<Result<DNSServiceRegisterReply>>(4);
        let tx = Box::into_raw(Box::new(tx));

        let mut raw: DNSServiceRef = null_mut();
        let result = DNSServiceRegister(
            &mut raw,
            0,
            0,
            c_name.map_or(null_mut(), |c| c.as_ptr()),
            service_type.as_ptr(),
            null(),
            null(),
            service.port.to_be(),
            txt_len,
            txt_record,
            Some(register_reply),
            tx as _,
        );
        if result == kDNSServiceErr_NoError {
            // process callback
            let shutdown_flag = Arc::new(AtomicBool::new(false));
            let raw_service = ServiceRef::new(raw, tx as _);

            // spin a thread that keeps the registration working
            run_thread_with_poll(raw_service, shutdown_flag.clone());

            let service = RegisteredDnsService { shutdown_flag };

            match rx.recv_timeout(CALLBACK_TIMEOUT) {
                Ok(Ok(_reply)) => Ok(service),
                Ok(Err(e)) => Err(e),
                Err(e) => {
                    error!("Error waiting for callback: {:?}", e);
                    Err(RegistrationError::ServiceError(0))
                }
            }
        } else {
            Err(RegistrationError::ServiceError(result))
        }
    }
}
