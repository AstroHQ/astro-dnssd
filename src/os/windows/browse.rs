use crate::browse::{Result, Service, ServiceEventType};
use crate::ffi::windows::{
    DNS_FREE_TYPE_DnsFreeRecordList, DnsFree, DnsServiceBrowse, DnsServiceBrowseCancel,
    _DNS_SERVICE_BROWSE_REQUEST__bindgen_ty_1 as BrowseCallbackUnion, DNS_QUERY_REQUEST_VERSION1,
    DNS_TYPE_A, DNS_TYPE_AAAA, DNS_TYPE_PTR, DNS_TYPE_SRV, DNS_TYPE_TEXT, DWORD, PDNS_RECORD,
    PVOID, _DNS_SERVICE_BROWSE_REQUEST, _DNS_SERVICE_CANCEL,
};
use crate::os::windows::to_utf16;
use crate::ServiceBrowserBuilder;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{Error as IoError, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;
use std::str::Utf8Error;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;
use thiserror::Error;
use widestring::{U16CStr, U16CString};
use winapi::shared::winerror::DNS_REQUEST_PENDING;

/// Error while browsing for DNS-SD services
#[derive(Debug, Error)]
pub enum BrowseError {
    /// Timeout waiting for more data, there may be no data available at this time
    #[error("Timeout waiting for data")]
    Timeout,
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
    /// Error from DnsService APIs
    #[error("Error from DNS Service APIs: {0}")]
    DnsError(DWORD),
    /// Error processing UTF8 string bytes from C API
    #[error("Error creating string from UTF8: {0}")]
    Utf8StringError(#[from] Utf8Error),
}
enum DnsRecord {
    Ptr(String),
    Srv(u16),
    Txt(HashMap<String, String>),
    A(Ipv4Addr),
    Aaaa(Ipv6Addr),
}

fn process_name(name: &str) -> Option<(String, String, String)> {
    // split doesn't do reverse so collect then reverse...
    let mut split = name.split(".").collect::<Vec<&str>>().into_iter().rev();
    let domain = split.next()?;
    let ip_protocol = split.next()?;
    let protocol = split.next()?;
    let name: String = split.collect::<Vec<&str>>().join(".");
    Some((name, format!("{}.{}", protocol, ip_protocol), domain.into()))
}

fn services_from_record_list(start_record: PDNS_RECORD) -> Result<Service> {
    let mut service = Service {
        name: "".to_string(),
        regtype: "".to_string(),
        interface_index: None,
        domain: "".to_string(),
        event_type: ServiceEventType::Added,
        hostname: "".to_string(),
        port: 0,
        txt_record: None,
    };
    let mut current_record = start_record;
    while !current_record.is_null() {
        match DnsRecord::try_from(current_record) {
            Ok(DnsRecord::Ptr(name)) => match process_name(&name) {
                Some((name, regtype, domain)) => {
                    service.hostname = format!("{}.{}", name, domain);
                    service.name = name;
                    service.regtype = regtype;
                    service.domain = domain;
                }
                None => {}
            },
            Ok(DnsRecord::Srv(port)) => service.port = port,
            Ok(DnsRecord::Txt(hash)) => {
                if hash.len() > 0 {
                    service.txt_record = Some(hash);
                }
            }
            Ok(DnsRecord::A(_ip)) => {}
            Ok(DnsRecord::Aaaa(_ip)) => {}
            Err(e) => {
                error!("Error processing DNS record, skipping it: {:?}", e);
            }
        }
        unsafe {
            current_record = (*current_record).pNext;
        }
    }
    Ok(service)
}

impl TryFrom<PDNS_RECORD> for DnsRecord {
    type Error = BrowseError;

    fn try_from(record: PDNS_RECORD) -> std::result::Result<Self, Self::Error> {
        if record.is_null() {
            return Err(IoError::from(ErrorKind::InvalidData).into());
        }
        let t = unsafe { (*record).wType } as u32; // what type of record
                                                   // let mut ptr_name = String::from("Unknown");
        match t {
            DNS_TYPE_PTR => unsafe {
                let data = (*record).Data.Ptr;
                let name = U16CString::from_ptr_str(data.pNameHost);
                let name = name.to_string_lossy();
                trace!("PTR Name: {}", name);
                Ok(DnsRecord::Ptr(name))
            },
            DNS_TYPE_SRV => unsafe {
                let data = (*record).Data.Srv;
                let port = data.wPort;
                trace!("Port: {}", port);
                Ok(DnsRecord::Srv(port))
            },
            DNS_TYPE_TEXT => unsafe {
                let strings = std::slice::from_raw_parts(
                    (*record).Data.Txt.pStringArray.as_ptr(),
                    (*record).Data.Txt.dwStringCount as _,
                );
                let mut hash = HashMap::with_capacity(strings.len());
                for str_ptr in strings {
                    match U16CStr::from_ptr_str(*str_ptr).to_string() {
                        Ok(s) => {
                            let mut split = s.split("=");
                            match (split.next(), split.next()) {
                                (Some(k), Some(v)) => {
                                    hash.insert(k.to_string(), v.to_string());
                                }
                                _ => {
                                    warn!("Failed to get key=value from TXT string: {}", s);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error parsing TXT string: {:?}", e);
                        }
                    }
                }

                Ok(DnsRecord::Txt(hash))
            },
            DNS_TYPE_A => unsafe {
                let data = (*record).Data.A;
                let ip = Ipv4Addr::from(data.IpAddress.to_le_bytes());
                trace!("IP Address: {}", ip);
                Ok(DnsRecord::A(ip))
            },
            DNS_TYPE_AAAA => unsafe {
                let data = (*record).Data.AAAA;
                let addr = data.Ip6Address; // TODO: bytes are wrong order here
                let ip = Ipv6Addr::from(addr.IP6Word);
                trace!("IPv6 Address: {}", ip);
                Ok(DnsRecord::Aaaa(ip))
            },
            _ => {
                warn!("Got record: {:?}, unhandled type", t);
                Err(IoError::from(ErrorKind::InvalidData).into())
            }
        }
    }
}

pub unsafe extern "C" fn browse_callback(status: DWORD, context: PVOID, record: PDNS_RECORD) {
    info!("Browse callback: {}", status);
    if status != 0 {
        error!("Error in callback: {}", status);
        return;
    }
    if context.is_null() {
        error!("Callback has nil context, returning early");
        return;
    }
    let tx_ptr: *mut SyncSender<Service> = context as _;
    let tx = &*tx_ptr;
    match services_from_record_list(record) {
        Ok(service) => {
            trace!("{:?}", service);
            match tx.send(service) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error sending service info: {:?}", e);
                }
            }
        }
        Err(e) => {
            error!("Error creating services from PDNS_RECORD: {:?}", e);
        }
    }
    DnsFree(record as _, DNS_FREE_TYPE_DnsFreeRecordList);
}
/// Service browser for DNS-SD services
pub struct ServiceBrowser {
    cancel: _DNS_SERVICE_CANCEL,
    context: *mut SyncSender<Service>,
    receiver: Receiver<Service>,
}
impl Drop for ServiceBrowser {
    fn drop(&mut self) {
        unsafe {
            let r = DnsServiceBrowseCancel(&mut self.cancel);
            if r != 0 {
                error!("Error canceling service browser: {}", r);
            }
            self.free_context();
        }
    }
}
impl ServiceBrowser {
    fn free_context(&mut self) {
        if !self.context.is_null() {
            unsafe { Box::from_raw(self.context) };
            self.context = null_mut();
        }
    }
    /// Receives any newly discovered services if any
    pub fn recv_timeout(&self, timeout: Duration) -> Result<Service> {
        match self.receiver.recv_timeout(timeout) {
            Ok(service) => Ok(service),
            Err(RecvTimeoutError::Timeout) => Err(BrowseError::Timeout),
            Err(RecvTimeoutError::Disconnected) => {
                Err(BrowseError::IoError(IoError::from(ErrorKind::BrokenPipe)))
            }
        }
    }
}
pub fn browse(builder: ServiceBrowserBuilder) -> Result<ServiceBrowser> {
    let name = format!("{}.local", builder.regtype);
    let mut name = to_utf16(name);
    let callback = BrowseCallbackUnion {
        pBrowseCallback: Some(browse_callback),
    };
    let (tx, rx) = sync_channel::<Service>(10);
    let tx = Box::into_raw(Box::new(tx));
    let mut request = _DNS_SERVICE_BROWSE_REQUEST {
        Version: DNS_QUERY_REQUEST_VERSION1,
        InterfaceIndex: 0,
        QueryName: name.as_mut_ptr(),
        __bindgen_anon_1: callback,
        pQueryContext: tx as _,
    };
    unsafe {
        let mut cancel: _DNS_SERVICE_CANCEL = std::mem::zeroed();
        let r = DnsServiceBrowse(&mut request, &mut cancel) as u32;
        if r != DNS_REQUEST_PENDING {
            return Err(BrowseError::DnsError(r));
        }
        Ok(ServiceBrowser {
            cancel,
            context: tx,
            receiver: rx,
        })
    }
}
