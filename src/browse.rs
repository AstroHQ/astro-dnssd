pub use crate::os::{BrowseError, ServiceBrowser};
use std::collections::HashMap;
#[cfg(target_vendor = "apple")]
use std::{net::SocketAddr, time::Duration};

/// Service browsing result type
pub type Result<T, E = BrowseError> = std::result::Result<T, E>;

/// Type of service event from browser, if a service is being added or removed from network
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    /// Index of the network interface the service was found on. On Apple platforms this is the interface it resolved
    /// on, so it's a real interface even for peer-to-peer services that browse reports as `kDNSServiceInterfaceIndexP2P`
    pub interface_index: Option<u32>,
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

#[cfg(target_vendor = "apple")]
impl Service {
    /// Looks up the service host's addresses through mDNSResponder on the interface the service resolved on, including
    /// peer-to-peer interfaces such as AWDL, which `getaddrinfo` doesn't search. IPv6 link-local addresses come back
    /// scoped to their interface. Returns the first batch of answers, waiting up to `timeout` for it. A host with only
    /// IPv4 or only IPv6 addresses is fine. Fails with `kDNSServiceErr_NoSuchRecord` if both are reported missing, or
    /// times out if nothing answers
    pub fn resolve_socket_addrs(&self, timeout: Duration) -> Result<Vec<SocketAddr>> {
        crate::os::resolve_socket_addrs(
            &self.hostname,
            self.port,
            self.interface_index.unwrap_or(0),
            timeout,
        )
    }
}

/// Builder for creating a browser, allowing optionally specifying a domain with chaining (maybe builder is excessive)
pub struct ServiceBrowserBuilder {
    pub(crate) regtype: String,
    pub(crate) domain: Option<String>,
    #[cfg(target_vendor = "apple")]
    pub(crate) peer_to_peer: bool,
}

impl ServiceBrowserBuilder {
    /// Creates new service browser for given service type, i.e. ._http._tcp
    pub fn new(regtype: &str) -> ServiceBrowserBuilder {
        ServiceBrowserBuilder {
            regtype: String::from(regtype),
            domain: None,
            #[cfg(target_vendor = "apple")]
            peer_to_peer: false,
        }
    }
    /// Adds a specified domain to browser's search
    pub fn with_domain(mut self, domain: &str) -> ServiceBrowserBuilder {
        self.domain = Some(String::from(domain));
        self
    }
    /// Also browses over peer-to-peer interfaces such as AWDL, so services can be found
    /// without both devices being on the same network. Off by default.
    ///
    /// Sets `kDNSServiceFlagsIncludeP2P | kDNSServiceFlagsIncludeAWDL`. Keeping AWDL active
    /// has a cost on Wi-Fi throughput/latency, so only enable it when needed.
    #[cfg(target_vendor = "apple")]
    pub fn with_peer_to_peer(mut self, enabled: bool) -> ServiceBrowserBuilder {
        self.peer_to_peer = enabled;
        self
    }
    /// Starts the browser
    pub fn browse(self) -> Result<ServiceBrowser> {
        crate::os::browse(self)
    }
}
