//! Astro DNS-SD - Rust wrapper crate for DNS-SD libraries

#![forbid(missing_docs)]

use thiserror::Error;

// pub mod browser;
mod ffi;
#[cfg(feature = "non-blocking")]
mod non_blocking;
mod os;
mod txt;

// pub mod register;
// pub mod txt;

pub use crate::os::{RegisteredDnsService, RegistrationError};
pub use txt::TxtRecord;

/// DNS-SD Service for registration use
pub struct DNSService {
    /// Type of service, like ._http._tcp.
    pub regtype: String,
    /// Name to advertise, sometimes name of device
    pub name: Option<String>,
    /// Domain, usually .local by default
    pub domain: Option<String>,
    /// Optional host, uses machine's default hostname by default
    pub host: Option<String>,
    /// Port service is listening on
    pub port: u16,
    /// TXT record for service if any
    pub txt: Option<TxtRecord>,
}
impl DNSService {
    /// Registers service, advertising it on the network
    pub fn register(self) -> Result<RegisteredDnsService> {
        os::register_service(self)
    }
}

#[macro_use]
extern crate log;

/// Common error for DNS-SD service
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum DNSServiceError {
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

/// Result type for dns-sd fallible returns
pub type Result<T, E = RegistrationError> = std::result::Result<T, E>;
