mod ffi;
pub mod browser;
pub mod register;

#[derive(Debug)]
pub enum DNSServiceError {
    InvalidString
}
