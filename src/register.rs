use crate::ffi::{DNSServiceErrorType, DNSServiceFlags, DNSServiceProcessResult, DNSServiceRef, DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister, DNSServiceUpdateRecord, kDNSServiceErr_NoError};
use crate::DNSServiceError;
use crate::txt::TXTRecord;
use std::ffi::{CString, CStr, c_void};
use std::os::raw::c_char;
use std::mem;
use std::ptr;

/// Builder for creating a new DNSService for registration purposes
pub struct DNSServiceBuilder {
    regtype: String,
    name: Option<String>,
    domain: Option<String>,
    host: Option<String>,
    port: u16,
    txt: Option<TXTRecord>,
}

/// DNS-SD Service for registration use
pub struct DNSService {
    pub regtype: String,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    pub txt: Option<TXTRecord>,
    raw: DNSServiceRef,
    reply_callback: Box<Fn(Result<DNSServiceRegisterReply, DNSServiceError>) -> ()>,
}

/// Reply information upon successful registration
#[derive(Debug)]
pub struct DNSServiceRegisterReply {
    pub regtype: String,
    pub name: String,
    pub domain: String,
}

/// Builder for creating a DNS-SD service to advertise
impl DNSServiceBuilder {
    /// Starts a new service builder with a given type (i.e. _http._tcp)
    pub fn new(regtype: &str) -> DNSServiceBuilder {
        DNSServiceBuilder {
            regtype: String::from(regtype),
            name: None,
            domain: None,
            host: None,
            port: 0,
            txt: None,
        }
    }

    /// Name to use for service, defaults to hostname
    pub fn with_name(mut self, name: &str) -> DNSServiceBuilder {
        self.name = Some(String::from(name));
        self
    }

    /// Domain to register service on, default is .local+
    pub fn with_domain(mut self, domain: &str) -> DNSServiceBuilder {
        self.domain = Some(String::from(domain));
        self
    }

    /// Host to use for service, defaults to machine's host
    pub fn with_host(mut self, host: &str) -> DNSServiceBuilder {
        self.host = Some(String::from(host));
        self
    }

    /// Port to use for service, 0 will mean a placeholder service, not showing up in browser
    pub fn with_port(mut self, port: u16) -> DNSServiceBuilder {
        self.port = port;
        self
    }

    /// Includes a TXT record for the service
    pub fn with_txt_record(mut self, txt: TXTRecord) -> DNSServiceBuilder {
        self.txt = Some(txt);
        self
    }

    /// Builds DNSService
    pub fn build(self) -> Result<DNSService, DNSServiceError> {
        unsafe {
            let service = DNSService {
                regtype: self.regtype,
                name: self.name,
                domain: self.domain,
                host: self.host,
                port: self.port,
                txt: self.txt,
                raw: mem::zeroed(),
                // TODO: replace this? think it might live forever
                reply_callback: Box::new(|_| {})
            };
            Ok(service)
        }
    }
}

impl DNSService {
    /// Get c_void ptr for use in C style context point arguments
    fn void_ptr(&mut self) -> *mut c_void {
        self as *mut _ as *mut c_void
    }

    /// Returns socket to mDNS service, use with select()
    pub fn socket(&self) -> i32 {
        unsafe {
            DNSServiceRefSockFD(self.raw)
        }
    }

    /// Processes a reply from mDNS service, blocking until there is one
    /// To avoid blocking, check if `socket` is ready for reading
    pub fn process_result(&self) -> DNSServiceErrorType {
        unsafe {
            DNSServiceProcessResult(self.raw)
        }
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

    unsafe extern "C" fn register_reply(_sd_ref: DNSServiceRef, _flags: DNSServiceFlags, error_code: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char, context: *mut c_void) {
        let context: &mut DNSService = &mut *(context as *mut DNSService);
        let process = || -> Result<(String, String, String), DNSServiceError> {
            let c_str: &CStr = CStr::from_ptr(name);
            let service_name: &str = c_str.to_str().map_err(|_| DNSServiceError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(regtype);
            let regtype: &str = c_str.to_str().map_err(|_| DNSServiceError::InternalInvalidString)?;
            let c_str: &CStr = CStr::from_ptr(domain);
            let reply_domain: &str = c_str.to_str().map_err(|_| DNSServiceError::InternalInvalidString)?;
            Ok((service_name.to_owned(), regtype.to_owned(), reply_domain.to_owned()))
        };
        match process() {
            Ok((name, regtype, domain)) => {
                if error_code == kDNSServiceErr_NoError {
                    let reply = DNSServiceRegisterReply {
                        regtype,
                        name,
                        domain,
                    };
                    (context.reply_callback)(Ok(reply));
                }
                else {
                    (context.reply_callback)(Err(DNSServiceError::ServiceError(error_code)));
                } 
            },
            Err(e) => {
                (context.reply_callback)(Err(e));
            },
        }
    }

    /// Registers service with mDNS responder, calling callback when a reply is received (requires calling process_result() when socket is ready)
    pub fn register<F: 'static>(&mut self, callback: F) -> Result<(), DNSServiceError>
        where F: Fn(Result<DNSServiceRegisterReply, DNSServiceError>) -> ()
    {
        // TODO: figure out if we can have non-'static callback
        self.reply_callback = Box::new(callback);
        unsafe {
            let c_name: Option<CString>;
            if let Some(n) = &self.name {
                c_name = Some(CString::new(n.as_str()).map_err(|_| DNSServiceError::InvalidString)?);
            } else {
                c_name = None;
            }
            let service_type = CString::new(self.regtype.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
            let (txt_record, txt_len) = match &mut self.txt {
                Some(txt) => (txt.get_bytes_ptr(), txt.len()),
                None => (ptr::null(), 0), 
            };
            let result = DNSServiceRegister(&mut self.raw, 0, 0, c_name.map_or(ptr::null_mut(), |c| c.as_ptr()), service_type.as_ptr(), 
                ptr::null(), ptr::null(), self.port.to_be(), txt_len, txt_record, Some(DNSService::register_reply), self.void_ptr());
            if result == kDNSServiceErr_NoError {
                return Ok(());
            }
            Err(DNSServiceError::ServiceError(result))
        }
    }

    /// Updates service's primary TXT record, removing it if provided None
    pub fn update_txt_record(&mut self, mut txt: Option<TXTRecord>) -> Result<(), DNSServiceError> {
        unsafe {
            let (txt_record, txt_len) = match &mut txt {
                Some(txt) => (txt.get_bytes_ptr(), txt.len()),
                None => (ptr::null(), 0), 
            };
            let result = DNSServiceUpdateRecord(self.raw, ptr::null_mut(), 0, txt_len, txt_record, 0);
            if result == kDNSServiceErr_NoError {
                return Ok(());
            }
            Err(DNSServiceError::ServiceError(result))
        }
    }
}

impl Drop for DNSService {
    fn drop(&mut self) {
        unsafe {
            DNSServiceRefDeallocate(self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let builder = DNSServiceBuilder::new("_http._tcp").with_port(5222);
        let mut service = builder.with_name("Blargh").build().unwrap();
        let reg_result = service.register(|reply| {
            println!("Reply: {:?}", reply);
        });
        assert_eq!(reg_result.is_ok(), true);
        let result = service.process_result();
        let socket = service.socket();
        assert_eq!(result, kDNSServiceErr_NoError);
        assert_ne!(socket, -1);
        assert_ne!(service.raw.is_null(), true);
    }
}
