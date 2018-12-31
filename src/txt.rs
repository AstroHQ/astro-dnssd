use crate::DNSServiceError;
use crate::ffi::{TXTRecordCreate, TXTRecordRef, TXTRecordSetValue, TXTRecordRemoveValue, TXTRecordDeallocate, TXTRecordGetLength, TXTRecordGetBytesPtr, TXTRecordContainsKey, kDNSServiceErr_NoError};
use std::ffi::{CString, c_void};
use std::mem;
use std::ptr;

/// Represents a TXTRecord for dns-sd, containing 0 or more key=value pairs
pub struct TXTRecord {
    raw: TXTRecordRef
}

impl TXTRecord {
    /// Creates a new empty TXTRecord with internally managed buffer
    pub fn new() -> TXTRecord {
        unsafe {
            let mut record = TXTRecord {
                raw: mem::uninitialized(),
            };
            TXTRecordCreate(&mut record.raw, 0, ptr::null_mut());
            record
        }
    }

    /// Sets a key/value pair with given strings
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), DNSServiceError> {
        self.set_value_bytes(key, value.as_bytes())
    }

    /// Sets a key/value pair to a raw bytes value
    pub fn set_value_bytes(&mut self, key: &str, value: &[u8]) -> Result<(), DNSServiceError> {
        unsafe {
            let key = CString::new(key).map_err(|_| DNSServiceError::InvalidString)?;
            let value_size = value.len() as u8;
            let result = TXTRecordSetValue(&mut self.raw, key.as_ptr(), value_size, value.as_ptr() as *mut c_void);
            if result == kDNSServiceErr_NoError {
                return Ok(());
            }
            Err(DNSServiceError::ServiceError(result))
        }
    }

    /// Removes a key/value pair
    pub fn remove_value(&mut self, key: &str) -> Result<(), DNSServiceError> {
        unsafe {
            let key = CString::new(key).map_err(|_| DNSServiceError::InvalidString)?;
            let result = TXTRecordRemoveValue(&mut self.raw, key.as_ptr());
            if result == kDNSServiceErr_NoError {
                return Ok(());
            }
            Err(DNSServiceError::ServiceError(result))
        }
    }

    /// Checks if a key exists
    pub fn contains_key(&mut self, key: &str) -> Result<bool, DNSServiceError> {
        unsafe {
            let key = CString::new(key).map_err(|_| DNSServiceError::InvalidString)?;
            return Ok(TXTRecordContainsKey(self.len(), self.get_bytes_ptr(), key.as_ptr()) == 1);
        }
    }

    /// Length in bytes of `TXTRecord` data
    pub fn len(&mut self) -> u16 {
        unsafe {
            TXTRecordGetLength(&mut self.raw)
        }
    }

    /// Raw bytes pointer for `TXTRecord`
    pub fn get_bytes_ptr(&mut self) -> *const c_void {
        unsafe {
            TXTRecordGetBytesPtr(&mut self.raw)
        }
    }
}

impl Drop for TXTRecord {
    fn drop(&mut self) {
        unsafe {
            TXTRecordDeallocate(&mut self.raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txt_creation() {
        let mut record = TXTRecord::new();
        let r = record.set_value("test", "value1");
        let len = record.len();
        assert_eq!(r.is_ok(), true);
        assert_eq!(len, 12);
        assert_eq!(record.contains_key("test").unwrap(), true);
    }
}