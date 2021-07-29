//! Astro DNS-SD - Rust wrapper crate for DNS-SD libraries

#![forbid(missing_docs)]

use thiserror::Error;

// pub mod browser;
mod ffi;
#[cfg(feature = "non-blocking")]
mod non_blocking;
mod os;
mod register;
// pub mod txt;

pub use crate::os::{RegisteredDnsService, RegistrationError};
pub use crate::register::DNSServiceBuilder;

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
