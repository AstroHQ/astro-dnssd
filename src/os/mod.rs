#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
mod windows;

#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
pub use windows::register_service;
#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
pub use windows::RegisteredDnsService;
#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
pub use windows::RegistrationError;

#[cfg(any(feature = "win-bonjour", not(target_os = "windows")))]
mod apple;
#[cfg(any(feature = "win-bonjour", not(target_os = "windows")))]
pub use apple::register_service;
