#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod ffi;

use crate::ffi::{DNSServiceErrorType, DNSServiceFlags, DNSServiceProcessResult, DNSServiceRef, DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister};
use std::ffi::{CString, CStr, c_void};
use std::os::raw::c_char;
use std::mem;
use std::ptr;

#[derive(Debug)]
pub enum DNSServiceError {
    InvalidString
}

pub struct DNSServiceBuilder {
    regtype: String,
    name: Option<String>,
    domain: Option<String>,
    host: Option<String>,
    port: u16,
}

pub struct DNSService {
    pub regtype: String,
    pub name: Option<String>,
    pub domain: Option<String>,
    pub host: Option<String>,
    pub port: u16,
    raw: ffi::DNSServiceRef,
    reply_callback: Box<Fn(u32, i32, &str, &str, &str) -> ()>,
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

    pub fn build(self) -> Result<DNSService, DNSServiceError> {
        unsafe {
            let service = DNSService {
                regtype: self.regtype,
                name: self.name,
                domain: self.domain,
                host: self.host,
                port: self.port,
                raw: mem::zeroed(),
                // TODO: replace this? think it might live forever
                reply_callback: Box::new(|_, _, _, _, _| {})
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
    pub fn process_result(&self) -> DNSServiceErrorType {
        unsafe {
            DNSServiceProcessResult(self.raw)
        }
    }

    unsafe extern "C" fn register_reply(_sdRef: DNSServiceRef, flags: DNSServiceFlags, errorCode: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char, context: *mut c_void) {
        let context: &mut DNSService = &mut *(context as *mut DNSService);
        // TODO: ensure the C string handling is safe
        let c_str: &CStr = CStr::from_ptr(name);
        let name: &str = c_str.to_str().unwrap();
        let c_str: &CStr = CStr::from_ptr(regtype);
        let regtype: &str = c_str.to_str().unwrap();
        let c_str: &CStr = CStr::from_ptr(domain);
        let domain: &str = c_str.to_str().unwrap();
        if errorCode == ffi::kDNSServiceErr_NoError {
            context.domain = Some(domain.to_owned());
            context.name = Some(name.to_owned());
        }
        (context.reply_callback)(flags, errorCode, name, regtype, domain);
    }

    pub fn register<F: 'static>(&mut self, callback: F) -> Result<(), DNSServiceError>
        where F: Fn(u32, i32, &str, &str, &str) -> ()
    {
        // TODO: figure out if we can have non-'static callback
        self.reply_callback = Box::new(callback);
        unsafe {
            let mut name: *const c_char = ptr::null_mut();
            // TODO: better way to manage CString lifetime here?
            let c_name: CString;
            if let Some(n) = &self.name {
                c_name = CString::new(n.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
                name = c_name.as_ptr();
            }
            let serviceType = CString::new(self.regtype.as_str()).map_err(|_| DNSServiceError::InvalidString)?;
            DNSServiceRegister(&mut self.raw as *mut _, 0, 0, name, serviceType.as_ptr(), 
                ptr::null(), ptr::null(), self.port, 0, ptr::null(), Some(DNSService::register_reply), self.void_ptr());
            Ok(())
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
    use super::ffi::kDNSServiceErr_NoError;

    #[test]
    fn it_works() {
        let builder = DNSServiceBuilder::new("_http._tcp").with_port(5222);
        let mut service = builder.with_name("Blargh").build().unwrap();
        let reg_result = service.register(|flags, error, name, regtype, domain| {
            println!("Flags: {}, Error: {}, Name: {}, Regtype: {}, Domain: {}", flags, error, name, regtype, domain);
        });
        assert_eq!(reg_result.is_ok(), true);
        let result = service.process_result();
        let socket = service.socket();
        assert_eq!(result, kDNSServiceErr_NoError);
        assert_ne!(socket, -1);
        assert_ne!(service.raw.is_null(), true);
    }
}
