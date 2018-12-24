#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod ffi {
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use ffi::{DNSServiceErrorType, DNSServiceFlags, DNSServiceProcessResult, DNSServiceRef, DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister};
use std::ffi::{CString, CStr, c_void};
use std::os::raw::c_char;
use std::mem;
use std::ptr;

#[derive(Debug)]
pub enum DNSServiceError {
    InvalidString
}

pub struct DNSServiceBuilder {
    regtype: String, // _http._tcp
    name: Option<String>, // MyHost
    domain: Option<String>,
    host: Option<String>,
    port: u16,
}

pub struct DNSService {
    pub regtype: String, // _http._tcp
    pub name: Option<String>, // MyHost
    pub domain: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    raw: ffi::DNSServiceRef,
}

impl DNSServiceBuilder {
    pub fn new(regtype: &str) -> DNSServiceBuilder {
        DNSServiceBuilder {
            regtype: String::from(regtype),
            name: None,
            domain: None,
            host: None,
            port: 0,
        }
    }

    pub fn with_name(mut self, name: &str) -> DNSServiceBuilder {
        self.name = Some(String::from(name));
        self
    }

    pub fn with_domain(mut self, domain: &str) -> DNSServiceBuilder {
        self.domain = Some(String::from(domain));
        self
    }

    pub fn with_host(mut self, host: &str) -> DNSServiceBuilder {
        self.host = Some(String::from(host));
        self
    }

    pub fn with_port(mut self, port: u16) -> DNSServiceBuilder {
        self.port = port;
        self
    }

    pub fn register(self) -> Result<DNSService, DNSServiceError> {
        unsafe {
            let mut service = DNSService {
                regtype: self.regtype,
                name: self.name,
                domain: self.domain,
                host: self.host,
                port: self.port,
                raw: mem::zeroed(),
            };
            let mut name: *const c_char = ptr::null_mut();
            let c_name: CString; // there's probably a better way to make it live longer?
            if let Some(n) = &service.name {
                c_name = CString::new(n.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
                name = c_name.as_ptr();
            }
            let serviceType = CString::new(service.regtype.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
            DNSServiceRegister(&mut service.raw as *mut _, 0, 0, name, serviceType.as_ptr(), 
                ptr::null(), ptr::null(), self.port, 0, ptr::null(), Some(DNSService::register_reply), service.void_ptr());
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
            return DNSServiceRefSockFD(self.raw);
        }
    }

    /// Processes a reply from mDNS service, blocking until there is one
    pub fn process_result(&self) -> DNSServiceErrorType {
        unsafe {
            return DNSServiceProcessResult(self.raw);
        }
    }

    fn process_register_reply(&self, flags: DNSServiceFlags, errorCode: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char) {
        let c_str: &CStr = unsafe { CStr::from_ptr(name) };
        let name: &str = c_str.to_str().unwrap();
        let c_str: &CStr = unsafe { CStr::from_ptr(regtype) };
        let regtype: &str = c_str.to_str().unwrap();
        let c_str: &CStr = unsafe { CStr::from_ptr(domain) };
        let domain: &str = c_str.to_str().unwrap();
        println!("Got reply, flags: {:?} error: {:?} name: {:?} regtype: {:?} domain: {:?}", flags, errorCode, name, regtype, domain);
    }

    unsafe extern "C" fn register_reply(_sdRef: DNSServiceRef, flags: DNSServiceFlags, errorCode: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char, context: *mut c_void) {
        let context: &mut DNSService = &mut *(context as *mut DNSService);
        context.process_register_reply(flags, errorCode, name, regtype, domain);
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
    use super::ffi::kDNSServiceErr_NoError;

    #[test]
    fn it_works() {
        let builder = DNSServiceBuilder::new("_http._tcp").with_port(5222);
        let service = builder.with_name("Blargh").register().unwrap();
        let result = service.process_result();
        let socket = service.socket();
        assert_eq!(result, kDNSServiceErr_NoError);
        assert_ne!(socket, -1);
        assert_ne!(service.raw.is_null(), true);
    }
}
