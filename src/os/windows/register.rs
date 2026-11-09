use crate::ffi::windows as ffi;
use crate::ffi::windows::{DWORD, PDNS_SERVICE_INSTANCE, PVOID};
use crate::os::windows::to_utf16;
use crate::DNSServiceBuilder;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::OsString;
use std::fmt;
use std::io::{Error as IoError, ErrorKind};
use std::os::windows::ffi::OsStringExt;
use std::ptr::null_mut;
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError, SyncSender};
use std::time::Duration;
use thiserror::Error;
use winapi::shared::winerror::DNS_REQUEST_PENDING;
use winapi::um::winbase::GetComputerNameW;

const CALLBACK_TIMEOUT: Duration = Duration::from_secs(10);

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

struct KeyValues {
    keys: Vec<Vec<u16>>,
    _values: Vec<Vec<u16>>,
    keys_ptr: Vec<*mut u16>,
    values_ptr: Vec<*mut u16>,
}

impl KeyValues {
    fn new(txt: &HashMap<String, String>) -> KeyValues {
        let mut keys = Vec::with_capacity(txt.len());
        let mut values = Vec::with_capacity(txt.len());
        for (key, value) in txt.iter() {
            keys.push(to_utf16(key));
            values.push(to_utf16(value));
        }
        let keys_ptr = keys.iter_mut().map(|k| k.as_mut_ptr()).collect();
        let values_ptr = values.iter_mut().map(|v| v.as_mut_ptr()).collect();
        KeyValues {
            keys,
            _values: values,
            keys_ptr,
            values_ptr,
        }
    }
}

trait DNSServiceExt {
    fn host_name(&self) -> String;
    fn service_name(&self) -> String;
}

impl DNSServiceExt for DNSServiceBuilder {
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

/// Builds the service instance to register. Returns null if it fails, see `GetLastError`
fn construct_instance(
    name: &str,
    host: &str,
    port: u16,
    txt: Option<&HashMap<String, String>>,
) -> *mut ffi::_DNS_SERVICE_INSTANCE {
    let mut name = to_utf16(name);
    let mut host = to_utf16(host);
    let mut kv_store = txt.map(KeyValues::new);
    let (property_count, keys_ptr, values_ptr) = match kv_store.as_mut() {
        Some(kv) => (
            kv.keys.len(),
            kv.keys_ptr.as_mut_ptr(),
            kv.values_ptr.as_mut_ptr(),
        ),
        None => (0, null_mut() as _, null_mut() as _),
    };
    // behavior suggests this copies it's arguments, so we can use pointers to rust stack here
    unsafe {
        ffi::DnsServiceConstructInstance(
            name.as_mut_ptr(),
            host.as_mut_ptr(),
            null_mut(),
            null_mut(),
            port,
            0,
            0,
            property_count as _,
            keys_ptr as _,
            values_ptr as _,
        )
    }
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

/// Takes ownership of the boxed sender in `context`. The caller can time out and drop the receiver before this runs,
/// so the box can only be freed here, and a failed send is fine.
unsafe extern "C" fn register_callback(
    status: DWORD,
    context: PVOID,
    instance: PDNS_SERVICE_INSTANCE,
) {
    if !context.is_null() {
        let tx = Box::from_raw(context as *mut SyncSender<DWORD>);
        trace!("Register complete: {} return code", status);
        _ = tx.send(status);
    }
    ffi::DnsServiceFreeInstance(instance);
}
enum State {
    /// not advertised
    Unregistered,
    /// advertised, or a de-register failed so it might still be
    Registered,
    /// `DnsServiceDeRegister` is pending but the callback timed out, so we keep the receiver to wait on it again
    Deregistering(Receiver<DWORD>),
}

/// Opaque type for a registered DNS-SD service, de-registering on drop
pub struct RegisteredDnsService {
    state: State,
    name: String,
    host: String,
    port: u16,
    request: ffi::_DNS_SERVICE_REGISTER_REQUEST,
    service: *mut ffi::_DNS_SERVICE_INSTANCE,
}
impl fmt::Debug for RegisteredDnsService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegisteredDnsService {{ {} {} }}", self.name, self.host)
    }
}
impl RegisteredDnsService {
    /// Replaces the service's TXT record. `DnsServiceRegister` can't change a registration, so this de-registers and
    /// registers again, and browsers see the service go away and come back.
    pub fn update_txt_record(&mut self, txt: HashMap<String, String>) -> Result<()> {
        let service = construct_instance(&self.name, &self.host, self.port, Some(&txt));
        if service.is_null() {
            let error = IoError::last_os_error();
            error!("Failed to create service: {:?}", error);
            return Err(error.into());
        }
        if let Err(e) = self.deregister() {
            unsafe { ffi::DnsServiceFreeInstance(service) };
            return Err(e);
        }
        unsafe { ffi::DnsServiceFreeInstance(self.service) };
        self.service = service;
        self.request.pServiceInstance = service;
        self.register()
    }

    /// De-registers and waits for it to finish, so registering again doesn't race it. If an earlier de-register timed
    /// out, this waits on that one again instead of sending another.
    fn deregister(&mut self) -> Result<()> {
        let rx = match std::mem::replace(&mut self.state, State::Unregistered) {
            State::Unregistered => return Ok(()),
            State::Deregistering(rx) => rx,
            State::Registered => match self.start_deregister() {
                Ok(rx) => rx,
                Err(e) => {
                    self.state = State::Registered;
                    return Err(e);
                }
            },
        };
        match rx.recv_timeout(CALLBACK_TIMEOUT) {
            Ok(0) => Ok(()),
            Ok(e) => {
                error!("De-registration callback returned error: {}", e);
                self.state = State::Registered;
                Err(RegistrationError::DnsStatusError(e))
            }
            Err(RecvTimeoutError::Timeout) => {
                error!("Timed out waiting for de-registration callback");
                self.state = State::Deregistering(rx);
                Err(
                    IoError::new(ErrorKind::TimedOut, "Timed out waiting for async callback")
                        .into(),
                )
            }

            // the callback always sends before it drops the sender, so this shouldn't happen
            Err(RecvTimeoutError::Disconnected) => {
                error!("De-registration callback dropped without a status");
                self.state = State::Registered;
                Err(IoError::new(ErrorKind::BrokenPipe, "Async callback disconnected").into())
            }
        }
    }

    fn start_deregister(&mut self) -> Result<Receiver<DWORD>> {
        let (tx, rx) = sync_channel::<DWORD>(1);
        self.request.pQueryContext = Box::into_raw(Box::new(tx)) as _;
        let result = unsafe { ffi::DnsServiceDeRegister(&mut self.request, null_mut()) };
        if result != DNS_REQUEST_PENDING {
            error!("Failed to de-register service: {}", result);
            self.free_context();
            return Err(IoError::from_raw_os_error(result as _).into());
        }

        // the callback owns the context now
        self.request.pQueryContext = null_mut();
        Ok(rx)
    }

    /// Only for when the call didn't return `DNS_REQUEST_PENDING`, otherwise `register_callback` frees it
    fn free_context(&mut self) {
        if !self.request.pQueryContext.is_null() {
            _ = unsafe { Box::from_raw(self.request.pQueryContext as *mut SyncSender<u32>) };
            self.request.pQueryContext = null_mut();
        }
    }
    fn register(&mut self) -> Result<()> {
        if !matches!(self.state, State::Unregistered) {
            warn!("Service already registered");
            return Ok(());
        }
        trace!(
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
        self.request.pQueryContext = null_mut();

        match rx.recv_timeout(CALLBACK_TIMEOUT) {
            Ok(0) => {
                // DNS_RCODE_NOERROR, from: https://docs.microsoft.com/en-us/windows/win32/dns/dns-constants#dns-response-codes
                self.state = State::Registered;
                Ok(())
            }
            Ok(e) => {
                error!("Registration callback returned error: {}", e);
                Err(RegistrationError::DnsStatusError(e))
            }
            Err(_e) => {
                error!("Timed out waiting for registration callback");
                Err(
                    IoError::new(ErrorKind::TimedOut, "Timed out waiting for async callback")
                        .into(),
                )
            }
        }
    }
}
impl TryFrom<DNSServiceBuilder> for RegisteredDnsService {
    type Error = std::io::Error;
    fn try_from(service: DNSServiceBuilder) -> Result<Self, Self::Error> {
        let name = service.service_name();
        let host = service.host_name();
        let instance = construct_instance(&name, &host, service.port, service.txt.as_ref());
        if instance.is_null() {
            let error = IoError::last_os_error();
            error!("Failed to create service: {:?}", error);
            return Err(error);
        }
        let request = ffi::_DNS_SERVICE_REGISTER_REQUEST {
            Version: ffi::DNS_QUERY_REQUEST_VERSION1,
            InterfaceIndex: 0, // 0 says all interfaces
            pServiceInstance: instance,
            pRegisterCompletionCallback: Some(register_callback),
            pQueryContext: null_mut(),
            hCredentials: null_mut(),
            unicastEnabled: 0, // false for mDNS protocol to advertise
        };
        Ok(RegisteredDnsService {
            name,
            host,
            port: service.port,
            state: State::Unregistered,
            request,
            service: instance,
        })
    }
}

impl Drop for RegisteredDnsService {
    fn drop(&mut self) {
        let deregister = match &self.state {
            State::Unregistered => false,
            State::Registered => true,

            // still in flight unless the callback came back with an error
            State::Deregistering(rx) => matches!(rx.try_recv(), Ok(status) if status != 0),
        };
        if deregister {
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
// should be safe to send across threads, just not access across
unsafe impl Send for RegisteredDnsService {}

pub fn register_service(service: DNSServiceBuilder) -> Result<RegisteredDnsService> {
    let mut service = RegisteredDnsService::try_from(service)?;
    service.register()?;
    Ok(service)
}
