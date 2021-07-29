use crate::os::{register_service, RegisteredDnsService};
use crate::Result;
use std::collections::HashMap;

/// Builder for creating a new DNSService for registration purposes
pub struct DNSServiceBuilder {
    pub(crate) regtype: String,
    pub(crate) name: Option<String>,
    pub(crate) domain: Option<String>,
    pub(crate) host: Option<String>,
    pub(crate) port: u16,
    pub(crate) txt: Option<HashMap<String, String>>,
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
    /// Registers service, advertising it on the network
    pub fn register(self) -> Result<RegisteredDnsService> {
        register_service(self)
    }
}
