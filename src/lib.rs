//! Astro DNS-SD - Rust wrapper crate for DNS-SD libraries

#![forbid(missing_docs)]

pub mod browser;
mod ffi;
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
