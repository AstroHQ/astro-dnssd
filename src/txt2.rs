//! TXT record creation & handling

use crate::ffi::apple::{
    kDNSServiceErr_NoError, TXTRecordContainsKey, TXTRecordCreate, TXTRecordDeallocate,
    TXTRecordGetBytesPtr, TXTRecordGetCount, TXTRecordGetLength, TXTRecordGetValuePtr,
    TXTRecordRef, TXTRecordRemoveValue, TXTRecordSetValue,
};
use crate::DNSServiceError;
use std::ffi::{c_void, CString};
use std::mem;
use std::ptr;
use std::slice;

/// Represents a TXT Record for dns-sd, containing 0 or more key=value pairs
pub struct TXTRecord {
    raw: TXTRecordRef,
}
impl Default for TXTRecord {
    fn default() -> Self {
        TXTRecord::new()
    }
}

impl TXTRecord {
    /// Creates a new empty TXT Record with an internally managed buffer
    pub fn new() -> TXTRecord {
        unsafe {
            let mut record = TXTRecord { raw: mem::zeroed() };
            TXTRecordCreate(&mut record.raw, 0, ptr::null_mut());
            record
        }
    }

    /// Sets a key/value pair
    ///
    /// **Note:** Only the first 256 bytes of the value will be used.
    pub fn insert<V>(&mut self, key: &str, value: Option<V>) -> Result<(), DNSServiceError>
    where
        V: AsRef<[u8]>,
    {
        let value = value.as_ref().map(|x| x.as_ref());
        let key = CString::new(key).or(Err(DNSServiceError::InvalidString))?;
        let value_size = value.map_or(0, |x| x.len().min(u8::max_value() as usize) as u8);
        let result = unsafe {
            TXTRecordSetValue(
                &mut self.raw,
                key.as_ptr(),
                value_size,
                value.map_or(ptr::null(), |x| x.as_ptr() as *const c_void),
            )
        };
        if result == kDNSServiceErr_NoError {
            return Ok(());
        }
        Err(DNSServiceError::ServiceError(result))
    }

    /// Removes a key/value pair
    pub fn remove(&mut self, key: &str) {
        let key = match CString::new(key) {
            Ok(key) => key,
            Err(_) => {
                // If the key contains an interior NUL then we know we don't contain it, and
                // therefore there's nothing to remove
                return;
            }
        };
        // NB: The only error that TXTRecordRemoveValue can return is one signifying that the key
        // did not exist.
        unsafe {
            TXTRecordRemoveValue(&mut self.raw, key.as_ptr());
        }
    }

    /// Checks if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        let key = match CString::new(key) {
            Ok(key) => key,
            Err(_) => {
                // If the key contains an interior NUL then we know we don't contain it
                return false;
            }
        };
        unsafe {
            TXTRecordContainsKey(self.raw_bytes_len(), self.raw_bytes_ptr(), key.as_ptr()) != 0
        }
    }

    /// Returns a reference to the value corresponding to the key
    ///
    /// If the TXT Record contains the key with a null value, this returns `None`. Use
    /// `contains_key()` to differentiate between a null value and the key not existing in the TXT
    /// Record.
    pub fn get(&self, key: &str) -> Option<&[u8]> {
        let key = match CString::new(key) {
            Ok(key) => key,
            Err(_) => {
                // If the key contains an interior NUL then we know we don't contain it
                return None;
            }
        };
        let mut value_len = 0u8;
        unsafe {
            let ptr = TXTRecordGetValuePtr(
                self.raw_bytes_len(),
                self.raw_bytes_ptr(),
                key.as_ptr(),
                &mut value_len,
            );
            if ptr.is_null() {
                None
            } else {
                Some(slice::from_raw_parts(ptr as *const u8, value_len as usize))
            }
        }
    }

    /// Returns the number of keys stored in the TXT Record
    pub fn len(&self) -> u16 {
        unsafe { TXTRecordGetCount(self.raw_bytes_len(), self.raw_bytes_ptr()) }
    }

    /// Returns if TXTRecord is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the raw bytes of the TXT Record as a slice
    pub fn raw_bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.raw_bytes_ptr() as *const u8,
                self.raw_bytes_len() as usize,
            )
        }
    }

    /// Returns the length in bytes of the TXT Record data
    pub fn raw_bytes_len(&self) -> u16 {
        unsafe { TXTRecordGetLength(&self.raw) }
    }

    /// Returns the raw bytes pointer for the TXT Record
    pub fn raw_bytes_ptr(&self) -> *const c_void {
        unsafe { TXTRecordGetBytesPtr(&self.raw) }
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
        assert_eq!(record.insert("test", Some("value1")), Ok(()));
        assert_eq!(record.len(), 1);
        assert_eq!(record.raw_bytes_len(), 12);
        assert_eq!(record.raw_bytes(), b"\x0Btest=value1");
        assert!(record.contains_key("test"));
        assert_eq!(record.get("test"), Some(&b"value1"[..]));
        assert_eq!(record.insert("test2", Some([1u8, 2, 3])), Ok(()));
        assert_eq!(record.len(), 2);
        assert_eq!(record.raw_bytes_len(), 22);
        assert_eq!(record.raw_bytes(), b"\x0Btest=value1\x09test2=\x01\x02\x03");
        assert!(record.contains_key("test2"));
        assert_eq!(record.get("test2"), Some(&[1u8, 2, 3][..]));
        assert_eq!(record.insert("test", None::<&str>), Ok(()));
        assert_eq!(record.len(), 2);
        assert_eq!(record.raw_bytes_len(), 15);
        assert_eq!(record.raw_bytes(), b"\x09test2=\x01\x02\x03\x04test");
        assert_eq!(record.get("test"), None);
    }
}
