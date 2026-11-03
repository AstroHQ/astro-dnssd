use crate::os::{register_service, RegisteredDnsService, RegistrationError};
use std::collections::HashMap;
pub type Result<T, E = RegistrationError> = std::result::Result<T, E>;

/// Builder for creating a new DNSService for registration purposes
pub struct DNSServiceBuilder {
    pub(crate) regtype: String,
    pub(crate) name: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) port: u16,
    pub(crate) txt: Option<HashMap<String, String>>,
    #[cfg_attr(
        all(target_os = "windows", not(feature = "win-bonjour")),
        allow(dead_code)
    )]
    pub(crate) interface_index: u32,
    #[cfg(target_vendor = "apple")]
    pub(crate) peer_to_peer: bool,
}
impl DNSServiceBuilder {
    /// Starts a new service builder with a given type (i.e. _http._tcp)
    pub fn new(regtype: &str, port: u16) -> DNSServiceBuilder {
        DNSServiceBuilder {
            regtype: String::from(regtype),
            name: None,
            domain: None,
            host: None,
            port,
            txt: None,
            interface_index: 0,
            #[cfg(target_vendor = "apple")]
            peer_to_peer: false,
        }
    }

    /// Name to use for service, defaults to hostname
    pub fn with_name(mut self, name: &str) -> DNSServiceBuilder {
        self.name = Some(String::from(name));
        self
    }

    /// Domain to register service on, default is .local+
    pub fn with_domain(mut self, domain: &str) -> DNSServiceBuilder {
        self.domain = Some(String::from(domain));
        self
    }

    /// Host to use for service, defaults to machine's host
    pub fn with_host(mut self, host: &str) -> DNSServiceBuilder {
        self.host = Some(String::from(host));
        self
    }

    /// Includes a TXT record for the service
    pub fn with_txt_record(mut self, txt: HashMap<String, String>) -> DNSServiceBuilder {
        self.txt = Some(txt);
        self
    }
    /// Adds key & value to TXT, creating a map if none yet
    pub fn with_key_value(mut self, key: String, value: String) -> DNSServiceBuilder {
        let mut kv = self.txt.take().unwrap_or_default();
        kv.insert(key, value);
        self.txt = Some(kv);
        self
    }
    /// Limits the registration to one network interface, given by its index.
    ///
    /// `0`, the default, registers on every interface. A registration covers a single interface,
    /// so a service that should appear on several of them needs one registration per index.
    /// Get the index of an interface from the platform (`if_nametoindex`, `getifaddrs`) or from
    /// [`Service::interface_index`](crate::Service::interface_index) of a browse result.
    ///
    /// This limits the service records (PTR, SRV, TXT). The address records of the host belong
    /// to the system and stay on every interface.
    ///
    /// Ignored on Windows with the native DNS-SD API, where the service is announced on every
    /// interface. In a test with two adapters, `DnsServiceRegister` sent probes and the goodbye on
    /// the chosen adapter only, but still sent its announcements on both.
    pub fn with_interface_index(mut self, index: u32) -> DNSServiceBuilder {
        self.interface_index = index;
        self
    }
    /// Also advertises over peer-to-peer interfaces such as AWDL, so peers can find the service
    /// without both devices being on the same network. Off by default.
    ///
    /// Sets `kDNSServiceFlagsIncludeP2P | kDNSServiceFlagsIncludeAWDL`. Note that for peers to
    /// actually connect over AWDL, the listening socket must also set `SO_RECV_ANYIF`.
    #[cfg(target_vendor = "apple")]
    pub fn with_peer_to_peer(mut self, enabled: bool) -> DNSServiceBuilder {
        self.peer_to_peer = enabled;
        self
    }
    /// Registers service, advertising it on the network
    pub fn register(self) -> Result<RegisteredDnsService> {
        register_service(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_index_defaults_to_all_interfaces() {
        assert_eq!(DNSServiceBuilder::new("_http._tcp", 80).interface_index, 0);
    }

    #[test]
    fn interface_index_is_stored() {
        let builder = DNSServiceBuilder::new("_http._tcp", 80).with_interface_index(7);
        assert_eq!(builder.interface_index, 7);
    }
}
