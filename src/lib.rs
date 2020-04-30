//! Astro DNS-SD - Rust wrapper crate for DNS-SD libraries

#![forbid(missing_docs)]

use std::error::Error;
use std::fmt;

pub mod browser;
mod ffi {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
pub mod register;
pub mod txt;

#[macro_use]
extern crate log;

/// Common error for DNS-SD service
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DNSServiceError {
    /// Invalid input string
    InvalidString,
    /// Unexpected invalid strings from C API
    InternalInvalidString,
    /// Error from DNSSD service
    ServiceError(i32),
}
impl fmt::Display for DNSServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DNSServiceError::InvalidString => {
                write!(f, "Invalid string argument, must be C-string compatible")
            }
            DNSServiceError::InternalInvalidString => write!(f, "Invalid string received from FFI"),
            DNSServiceError::ServiceError(e) => write!(f, "Service error: {}", e),
        }
    }
}
impl Error for DNSServiceError {
    // fn source(&self) -> Option<&(dyn Error + 'static)> { }
}
