use crate::browse::{Result, Service, ServiceEventType};
use crate::ffi::windows::{
    DNS_FREE_TYPE_DnsFreeRecordList, DnsFree, DnsServiceBrowse, DnsServiceBrowseCancel,
    _DNS_SERVICE_BROWSE_REQUEST__bindgen_ty_1 as BrowseCallbackUnion, DNS_QUERY_REQUEST_VERSION1,
    DNS_STATUS, DNS_TYPE_A, DNS_TYPE_AAAA, DNS_TYPE_PTR, DNS_TYPE_SRV, DNS_TYPE_TEXT, DWORD,
    PDNS_RECORD, PVOID, _DNS_SERVICE_BROWSE_REQUEST, _DNS_SERVICE_CANCEL,
};
use crate::os::windows::to_utf16;
use crate::ServiceBrowserBuilder;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::io::{Error as IoError, ErrorKind};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::ptr::null_mut;
use std::str::Utf8Error;
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use thiserror::Error;
use widestring::U16CString;
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

fn services_from_record_list(start_record: PDNS_RECORD) -> Result<Vec<Service>> {
    let mut services = vec![];
    let mut current_record = start_record;
    while !current_record.is_null() {
        let service = Service::try_from(current_record)?;
        services.push(service);
        unsafe {
            current_record = (*current_record).pNext;
        }
    }
    Ok(services)
}

impl TryFrom<PDNS_RECORD> for Service {
    type Error = BrowseError;

    fn try_from(record: PDNS_RECORD) -> std::result::Result<Self, Self::Error> {
        if record.is_null() {
            return Err(IoError::from(ErrorKind::InvalidData).into());
        }
        let t = unsafe { (*record).wType } as u32; // what type of record
        let mut ptr_name = String::from("Unknown");
        match t {
            DNS_TYPE_PTR => unsafe {
                let data = (*record).Data.Ptr;
                let name = U16CString::from_ptr_str(data.pNameHost);
                let name = name.to_string_lossy();
                info!("PTR Name: {}", name);
                ptr_name = name;
            },
            DNS_TYPE_SRV => unsafe {
                let data = (*record).Data.Srv;
                let port = data.wPort;
                info!("Port: {}", port);
            },
            DNS_TYPE_TEXT => unsafe {
                let data = (*record).Data.Txt;
                info!("TXT records: {}", data.dwStringCount);
            },
            DNS_TYPE_A => unsafe {
                let data = (*record).Data.A;
                let ip = Ipv4Addr::from(data.IpAddress.to_le_bytes());
                info!("IP Address: {}", ip);
            },
            DNS_TYPE_AAAA => unsafe {
                let data = (*record).Data.AAAA;
                let addr = data.Ip6Address; // TODO: bytes are wrong order here
                let ip = Ipv6Addr::from(addr.IP6Word);
                info!("IPv6 Address: {}", ip);
            },
            _ => {
                warn!("Got record: {:?}, unhandled type", t);
            }
        }

        let regtype = unsafe { U16CString::from_ptr_str((*record).pName) };
        Ok(Service {
            name: ptr_name,
            regtype: regtype.to_string_lossy(),
            interface_index: 0,
            domain: "".to_string(),
            event_type: ServiceEventType::Added,
        })
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
        Ok(services) => {
            info!("Services: {:?}", services);
            // match tx.send(service) {
            //     Ok(_) => {}
            //     Err(e) => {
            //         error!("Error sending service info: {:?}", e);
            //     }
            // }
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
