mod register;
mod txt;
pub use register::{register_service, RegisteredDnsService};
use thiserror::Error;

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
