pub use crate::os::{BrowseError, ServiceBrowser};
use std::collections::HashMap;
// use std::io::Error as IoError;
// use thiserror::Error;

/// Service browsing result type
pub type Result<T, E = BrowseError> = std::result::Result<T, E>;

pub trait ServiceBrowserTrait {
    fn next_entry(&self) -> Result<Service>;
}

/// Type of service event from browser, if a service is being added or removed from network
#[derive(Debug)]
pub enum ServiceEventType {
    /// Service has been added to the network
    Added,
    /// Service is removed from the network
    Removed,
}

/// Encapsulates information about a service
#[derive(Debug)]
pub struct Service {
    /// Name of service, usually a user friendly name
    pub name: String,
    /// Registration type, i.e. _http._tcp.
    pub regtype: String,
    /// Interface index (unsure what this is for)
    pub interface_index: u32,
    /// Domain service is on, typically local.
    pub domain: String,
    /// Whether this service is being added or not
    pub event_type: ServiceEventType,
    // /// Full name of service
    // pub full_name: String,
    /// Hostname of service, usable with gethostbyname()
    pub hostname: String,
    /// Port service is on
    pub port: u16,
    /// TXT record service has if any
    pub txt_record: Option<HashMap<String, String>>,
}

/// Builder for creating a browser, allowing optionally specifying a domain with chaining (maybe builder is excessive)
pub struct ServiceBrowserBuilder {
    pub(crate) regtype: String,
    pub(crate) domain: Option<String>,
}

impl ServiceBrowserBuilder {
    /// Creates new service browser for given service type, i.e. ._http._tcp
    pub fn new(regtype: &str) -> ServiceBrowserBuilder {
        ServiceBrowserBuilder {
            regtype: String::from(regtype),
            domain: None,
        }
    }
    /// Adds a specified domain to browser's search
    pub fn with_domain(mut self, domain: &str) -> ServiceBrowserBuilder {
        self.domain = Some(String::from(domain));
        self
    }
    /// Starts the browser
    pub fn browse(self) -> Result<ServiceBrowser> {
        crate::os::browse(self)
    }
}
