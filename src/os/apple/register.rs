//! Registration of dns-sd services

use super::txt::TXTRecord;
use crate::ffi::apple::{
    kDNSServiceErr_NoError, DNSServiceErrorType, DNSServiceFlags, DNSServiceProcessResult,
    DNSServiceRef, DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister,
    DNSServiceUpdateRecord,
};
use crate::{register::Result, DNSServiceBuilder};
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::os::raw::c_char;
use std::ptr;
use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::time::Duration;
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
                    tx.send((Ok(reply))).unwrap();
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
    raw: DNSServiceRef,
}
impl fmt::Debug for RegisteredDnsService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegisteredDnsService {{ raw: {:p} }}", self.raw)
    }
}

/// Reply information upon successful registration
#[derive(Debug)]
pub struct DNSServiceRegisterReply {
    /// Service type of successfully registered service
    pub regtype: String,
    /// Name of service
    pub name: String,
    /// Domain used for successful registration
    pub domain: String,
}

impl RegisteredDnsService {
    // /// Get c_void ptr for use in C style context point arguments
    // fn void_ptr(&mut self) -> *mut c_void {
    //     self as *mut _ as *mut c_void
    // }

    /// Returns socket to mDNS service, use with select()
    pub fn socket(&self) -> i32 {
        unsafe { DNSServiceRefSockFD(self.raw) }
    }

    /// Processes a reply from mDNS service, blocking until there is one
    /// To avoid blocking, check if `socket` is ready for reading
    pub fn process_result(&self) -> DNSServiceErrorType {
        unsafe { DNSServiceProcessResult(self.raw) }
    }

    // /// returns true if the socket has data and process_result() should be called
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

    // /// Updates service's primary TXT record, removing it if provided None
    // pub fn update_txt_record(&mut self, mut txt: Option<TXTRecord>) -> Result<()> {
    //     unsafe {
    //         let (txt_record, txt_len) = match &mut txt {
    //             Some(txt) => (txt.raw_bytes_ptr(), txt.raw_bytes_len()),
    //             None => (ptr::null(), 0),
    //         };
    //         let result =
    //             DNSServiceUpdateRecord(self.raw, ptr::null_mut(), 0, txt_len, txt_record, 0);
    //         if result == kDNSServiceErr_NoError {
    //             return Ok(());
    //         }
    //         Err(DNSServiceError::ServiceError(result))
    //     }
    // }
}

impl Drop for RegisteredDnsService {
    fn drop(&mut self) {
        unsafe {
            DNSServiceRefDeallocate(self.raw);
        }
    }
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
        // let (txt_record, txt_len) = match &mut service.txt {
        //     Some(txt) => (txt.raw_bytes_ptr(), txt.raw_bytes_len()),
        //     None => (ptr::null(), 0),
        // };
        let (tx, rx) = sync_channel::<Result<DNSServiceRegisterReply>>(1);
        let tx = Box::into_raw(Box::new(tx));

        let mut raw: DNSServiceRef = null_mut();
        let result = DNSServiceRegister(
            &mut raw,
            0,
            0,
            c_name.map_or(ptr::null_mut(), |c| c.as_ptr()),
            service_type.as_ptr(),
            ptr::null(),
            ptr::null(),
            service.port.to_be(),
            0,
            ptr::null_mut(),
            Some(register_reply),
            tx as _,
        );
        if result == kDNSServiceErr_NoError {
            // process callback
            let service = RegisteredDnsService { raw };
            let r = service.process_result();
            if r == kDNSServiceErr_NoError {
                match rx.recv_timeout(CALLBACK_TIMEOUT) {
                    Ok(Ok(reply)) => Ok(RegisteredDnsService { raw }),
                    Ok(Err(e)) => Err(e),
                    Err(e) => {
                        error!("Error waiting for callback: {:?}", e);
                        Err(RegistrationError::ServiceError(0))
                    }
                }
            } else {
                Err(RegistrationError::ServiceError(r))
            }
        } else {
            Err(RegistrationError::ServiceError(result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_with_name() {
        use std::sync::mpsc::channel;
        use std::time::Duration;
        let (tx, rx) = channel::<bool>();
        let builder = DNSServiceBuilder::new("_http._tcp", 5222);
        let mut service = builder.with_name("MyRustService").register().unwrap();
        // let reg_result = service.register(move |reply| {
        //     assert!(reply.is_ok());
        //     let reply = reply.unwrap();
        //     assert_eq!(reply.regtype, "_http._tcp.");
        //     assert_eq!(reply.name, "MyRustService");
        //     assert_eq!(reply.domain, "local.");
        //     tx.send(true).unwrap();
        // });
        // should have a raw pointer & register result should be Ok
        assert_ne!(service.raw.is_null(), true);
        // assert_eq!(reg_result.is_ok(), true);
        // This should block until we get a reply, and return no error once it does
        // let result = service.process_result();
        // assert_eq!(result, kDNSServiceErr_NoError);
        // ensure we get the reply (we saw it failing on linux)
        // let d = Duration::from_millis(500);
        // let reply_happened = rx.recv_timeout(d).unwrap();
        // assert_eq!(reply_happened, true);
        // should have a valid socket
        // let socket = service.socket();
        // assert_ne!(socket, -1);
    }

    // #[test]
    // fn registration_with_txt() {
    //     let txt = TXTRecord::new();
    // }
}
