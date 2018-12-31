mod ffi;
pub mod browser;
pub mod register;
pub mod txt;

#[derive(Debug)]
pub enum DNSServiceError {
    /// Invalid input string
    InvalidString,
    /// Unexpected invalid strings from C API
    InternalInvalidString,
    /// Error from DNSSD service
    ServiceError(i32),
}
