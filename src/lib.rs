//! Astro DNS-SD - Rust wrapper crate for DNS-SD libraries

#![forbid(missing_docs)]

// pub mod browser;
mod browse;
mod ffi;
mod non_blocking;
mod os;
mod register;

pub use crate::browse::{BrowseError, ServiceBrowser, ServiceBrowserBuilder};
pub use crate::os::{RegisteredDnsService, RegistrationError};
pub use crate::register::DNSServiceBuilder;

#[macro_use]
extern crate log;

// /// Result type for dns-sd fallible returns
// pub type Result<T, E = RegistrationError> = std::result::Result<T, E>;
