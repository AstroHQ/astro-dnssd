use crate::DNSServiceError;
use crate::ffi::{TXTRecordCreate, TXTRecordRef, TXTRecordSetValue, TXTRecordRemoveValue, TXTRecordDeallocate, TXTRecordGetLength, TXTRecordGetBytesPtr, kDNSServiceErr_NoError};
use std::ffi::{CString, c_void};
use std::mem;
use std::ptr;

pub struct TXTRecord {
    raw: TXTRecordRef
}

impl TXTRecord {
    pub fn new() -> TXTRecord {
        unsafe {
            let mut record = TXTRecord {
                raw: mem::uninitialized(),
            };
            TXTRecordCreate(&mut record.raw, 0, ptr::null_mut());
            record
        }
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<(), DNSServiceError> {
        unsafe {
            let key = CString::new(key).map_err(|_| DNSServiceError::InvalidString)?;
            let value_size = value.len() as u8;
            let value = CString::new(value).map_err(|_| DNSServiceError::InvalidString)?;
            let result = TXTRecordSetValue(&mut self.raw, key.as_ptr(), value_size, value.as_ptr() as *mut c_void);
            if result == kDNSServiceErr_NoError {
                return Ok(());
            }
            Err(DNSServiceError::ServiceError(result))
        }
    }

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

    pub fn len(&mut self) -> u16 {
        unsafe {
            TXTRecordGetLength(&mut self.raw)
        }
    }

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
    // use crate::ffi::kDNSServiceErr_NoError;

    #[test]
    fn txt_creation() {
        let mut record = TXTRecord::new();
        let r = record.set_value("test", "value1");
        let len = record.len();
        assert_eq!(r.is_ok(), true);
        assert_eq!(len, 12);
    }
}