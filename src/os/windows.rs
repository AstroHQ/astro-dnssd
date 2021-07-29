use crate::ffi::windows as ffi;
use crate::ffi::windows::{DWORD, PDNS_SERVICE_INSTANCE, PVOID};
use crate::DNSService;
use std::convert::TryFrom;
use std::ffi::OsString;
use std::fmt;
use std::io::{Error as IoError, ErrorKind};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::time::Duration;
use thiserror::Error;
use winapi::shared::winerror::DNS_REQUEST_PENDING;
use winapi::um::winbase::GetComputerNameW;

/// Errors during DNS-SD registration
#[derive(Debug, Error)]
pub enum RegistrationError {
    /// IO Error
    #[error("IO Error: {0}")]
    IoError(#[from] IoError),
    /// Error occurred during registration, non-successful DNS return code
    #[error("DNS return code error: {0}")]
    DnsStatusError(DWORD),
}

/// Registration result type
pub type Result<T, E = RegistrationError> = std::result::Result<T, E>;

trait DNSServiceExt {
    fn host_name(&self) -> String;
    fn service_name(&self) -> String;
}

impl DNSServiceExt for DNSService {
    fn host_name(&self) -> String {
        let host = self
            .host
            .clone()
            .or(computer_name())
            .unwrap_or_else(|| String::from("Unknown"));
        format!("{}.local", host)
    }

    fn service_name(&self) -> String {
        let name = self
            .name
            .clone()
            .or(computer_name())
            .unwrap_or_else(|| String::from("Unknown"));
        format!("{}.{}.local", name, self.regtype)
    }
}

pub fn to_utf16<S: AsRef<std::ffi::OsStr>>(s: S) -> Vec<u16> {
    s.as_ref()
        .encode_wide()
        .chain(Some(0u16).into_iter())
        .collect()
}
fn computer_name() -> Option<String> {
    unsafe {
        let mut buf = vec![0u16; 1024];
        let mut len = buf.len() as u32;
        if GetComputerNameW(buf.as_mut_ptr(), &mut len) != 0 {
            return Some(
                OsString::from_wide(&buf[0..len as usize])
                    .into_string()
                    .unwrap(),
            );
        }
    }
    None
}

unsafe extern "C" fn register_callback(
    status: DWORD,
    context: PVOID,
    instance: PDNS_SERVICE_INSTANCE,
) {
    if !context.is_null() {
        let tx_ptr: *mut SyncSender<DWORD> = context as _;
        let tx = &*tx_ptr;
        println!("Register complete: {}", status);
        tx.send(status).unwrap();
    }
    ffi::DnsServiceFreeInstance(instance);
}
/// Opaque type for a registered DNS-SD service, de-registering on drop
pub struct RegisteredDnsService {
    registered: bool,
    name: String,
    host: String,
    request: ffi::_DNS_SERVICE_REGISTER_REQUEST,
    service: *mut ffi::_DNS_SERVICE_INSTANCE,
}
impl fmt::Debug for RegisteredDnsService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegisteredDnsService {{ {} {} }}", self.name, self.host)
    }
}
impl RegisteredDnsService {
    fn free_context(&mut self) {
        if !self.request.pQueryContext.is_null() {
            unsafe { Box::from_raw(self.request.pQueryContext) };
            self.request.pQueryContext = null_mut();
        }
    }
    fn register(&mut self) -> Result<()> {
        if self.registered {
            warn!("Service already registered");
            return Ok(());
        }
        info!(
            "Registering:  name: {} host: {} port: {}",
            self.name,
            self.host,
            unsafe { (*self.service).wPort }
        );

        let (tx, rx) = sync_channel::<DWORD>(1);
        let tx = Box::into_raw(Box::new(tx));
        self.request.pQueryContext = tx as _;
        let result = unsafe { ffi::DnsServiceRegister(&mut self.request, std::ptr::null_mut()) };
        if result != DNS_REQUEST_PENDING {
            error!("Failed to register: {}", result);
            self.free_context();
            return Err(IoError::from_raw_os_error(result as _).into());
        }

        match rx.recv_timeout(Duration::from_secs(10)) {
            Ok(0) => {
                self.free_context();
                self.registered = true;
                Ok(())
            }
            Ok(e) => {
                self.free_context();
                Err(RegistrationError::DnsStatusError(e))
            }
            Err(_e) => {
                self.free_context();
                Err(
                    IoError::new(ErrorKind::TimedOut, "Timed out waiting for async callback")
                        .into(),
                )
            }
        }
    }
}
impl TryFrom<DNSService> for RegisteredDnsService {
    type Error = std::io::Error;
    fn try_from(service: DNSService) -> Result<Self, Self::Error> {
        unsafe {
            let original_name = service.service_name();
            let original_host = service.host_name();
            let mut name = to_utf16(&original_name);
            let mut host = to_utf16(&original_host);
            let service = ffi::DnsServiceConstructInstance(
                name.as_mut_ptr(),
                host.as_mut_ptr(),
                null_mut(),
                null_mut(),
                service.port,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
            );
            let request = ffi::_DNS_SERVICE_REGISTER_REQUEST {
                Version: ffi::DNS_QUERY_REQUEST_VERSION1,
                InterfaceIndex: 0, // 0 says all interfaces
                pServiceInstance: service,
                pRegisterCompletionCallback: Some(register_callback),
                pQueryContext: null_mut(),
                hCredentials: null_mut(),
                unicastEnabled: 0, // false for mDNS protocol to advertise
            };
            Ok(RegisteredDnsService {
                name: original_name,
                host: original_host,
                registered: false,
                request,
                service,
            })
        }
    }
}

impl Drop for RegisteredDnsService {
    fn drop(&mut self) {
        if self.registered {
            trace!("De-registering service...");
            let r = unsafe { ffi::DnsServiceDeRegister(&mut self.request, std::ptr::null_mut()) };
            if r != DNS_REQUEST_PENDING {
                error!("Failed to de-register service: {}", r);
            }
        }

        if !self.service.is_null() {
            trace!("Freeing service");
            unsafe { ffi::DnsServiceFreeInstance(self.service) };
            self.service = std::ptr::null_mut();
        }
    }
}

pub fn register_service(service: DNSService) -> Result<RegisteredDnsService> {
    let mut service = RegisteredDnsService::try_from(service).unwrap();
    service.register()?;
    Ok(service)
}
