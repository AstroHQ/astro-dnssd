use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxtRecordError {
    /// Provided key string is too long for DNS-SD API
    #[error("Key string exceeds length limit")]
    KeyTooLong,
    /// Provided value string is too long for DNS-SD API
    #[error("Value string exceeds length limit")]
    ValueTooLong,
}
pub type Result<T, E = TxtRecordError> = std::result::Result<T, E>;

/// Key-value record for DNS-SD services
pub struct TxtRecord {
    properties: HashMap<String, String>,
}
impl TxtRecord {
    /// Creates an empty TxtRecord
    pub fn new() -> Self {
        TxtRecord {
            properties: HashMap::new(),
        }
    }
    /// Inserts a key-value pair into TxtRecord
    pub fn insert<K: ToString, V: ToString>(&mut self, key: K, value: V) -> Result<()> {
        self.properties.insert(key.to_string(), value.to_string());
        Ok(())
    }
    /// Removes given key if any from record
    pub fn remove<K: ToString>(&mut self, key: K) {
        self.properties.remove(&key.to_string());
    }
}
