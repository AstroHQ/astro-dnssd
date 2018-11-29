#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod ffi {
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use ffi::{DNSServiceErrorType, DNSServiceFlags, DNSServiceProcessResult, DNSServiceRef, DNSServiceRefDeallocate, DNSServiceRefSockFD, DNSServiceRegister};
use std::ffi::{CString, c_void};
use std::os::raw::c_char;
use std::mem;
use std::ptr;

pub struct Service {
    raw: ffi::DNSServiceRef,
}
impl Service {
    
    pub fn register(name: &str, serviceType: &str) -> Service {
        unsafe {
            let mut service = Service { raw: mem::zeroed() };
            let name = CString::new(name).expect("CString::new failed");
            let serviceType = CString::new(serviceType).expect("CString::new failed");
            DNSServiceRegister(&mut service.raw as *mut _, 0, 0, name.as_ptr(), serviceType.as_ptr(), ptr::null(), ptr::null(), 2048, 0, ptr::null(), Some(Service::register_reply), ptr::null_mut());
            service
        }
    }

    pub fn socket(&self) -> i32 {
        unsafe {
            return DNSServiceRefSockFD(self.raw);
        }
    }

    pub fn process_result(&self) -> DNSServiceErrorType {
        unsafe {
            return DNSServiceProcessResult(self.raw);
        }
    }

    fn process_register_reply(&self, flags: DNSServiceFlags, errorCode: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char) {
        println!("Got reply, flags: {:?} error: {:?} name: {:?} regtype: {:?} domain: {:?}", flags, errorCode, name, regtype, domain);
    }

    unsafe extern "C" fn register_reply(_sdRef: DNSServiceRef, flags: DNSServiceFlags, errorCode: DNSServiceErrorType, name: *const c_char, regtype: *const c_char, domain: *const c_char, context: *mut c_void) {
        let context: &mut Service = &mut *(context as *mut Service);
        context.process_register_reply(flags, errorCode, name, regtype, domain);
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        println!("Dropping!");
        unsafe {
            DNSServiceRefDeallocate(self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::mem;
    // use std::ffi::{CString, c_void};
    // use std::os::raw::c_char;
    // use std::ptr;
    use super::ffi::kDNSServiceErr_NoError;

    #[test]
    fn it_works() {
        let service = Service::register("Blargh", "_http._tcp");
        let result = service.process_result();
        let socket = service.socket();
        assert_eq!(result, kDNSServiceErr_NoError);
        assert_ne!(socket, -1);
    }
}
