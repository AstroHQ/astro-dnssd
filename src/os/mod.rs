#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
mod windows;

#[cfg(all(not(feature = "win-bonjour"), target_os = "windows"))]
pub use windows::{
    browse::{browse, BrowseError, ServiceBrowser},
    register::{register_service, RegisteredDnsService, RegistrationError},
};

#[cfg(any(feature = "win-bonjour", not(target_os = "windows")))]
mod apple;
#[cfg(any(feature = "win-bonjour", not(target_os = "windows")))]
pub use apple::{register_service, RegisteredDnsService, RegistrationError};
