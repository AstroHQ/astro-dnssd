/* automatically generated by rust-bindgen */

#![allow(non_camel_case_types, non_snake_case, dead_code)]

pub const DNS_QUERY_REQUEST_VERSION1: u32 = 1;
pub type wchar_t = ::std::os::raw::c_ushort;
pub type ULONG = ::std::os::raw::c_ulong;
pub type DWORD = ::std::os::raw::c_ulong;
pub type BOOL = ::std::os::raw::c_int;
pub type BYTE = ::std::os::raw::c_uchar;
pub type WORD = ::std::os::raw::c_ushort;
pub type PVOID = *mut ::std::os::raw::c_void;
pub type WCHAR = wchar_t;
pub type LPWSTR = *mut WCHAR;
pub type PWSTR = *mut WCHAR;
pub type PCWSTR = *const WCHAR;
pub type HANDLE = *mut ::std::os::raw::c_void;
pub type QWORD = ::std::os::raw::c_ulonglong;
pub type IP4_ADDRESS = DWORD;
pub type PIP4_ADDRESS = *mut DWORD;
#[repr(C)]
#[derive(Copy, Clone)]
pub union IP6_ADDRESS {
    pub IP6Qword: [QWORD; 2usize],
    pub IP6Dword: [DWORD; 4usize],
    pub IP6Word: [WORD; 8usize],
    pub IP6Byte: [BYTE; 16usize],
    _bindgen_union_align: [u64; 2usize],
}
#[test]
fn bindgen_test_layout_IP6_ADDRESS() {
    assert_eq!(
        ::std::mem::size_of::<IP6_ADDRESS>(),
        16usize,
        concat!("Size of: ", stringify!(IP6_ADDRESS))
    );
    assert_eq!(
        ::std::mem::align_of::<IP6_ADDRESS>(),
        8usize,
        concat!("Alignment of ", stringify!(IP6_ADDRESS))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<IP6_ADDRESS>())).IP6Qword as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(IP6_ADDRESS),
            "::",
            stringify!(IP6Qword)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<IP6_ADDRESS>())).IP6Dword as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(IP6_ADDRESS),
            "::",
            stringify!(IP6Dword)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<IP6_ADDRESS>())).IP6Word as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(IP6_ADDRESS),
            "::",
            stringify!(IP6Word)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<IP6_ADDRESS>())).IP6Byte as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(IP6_ADDRESS),
            "::",
            stringify!(IP6Byte)
        )
    );
}
pub type PIP6_ADDRESS = *mut IP6_ADDRESS;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DNS_SERVICE_INSTANCE {
    pub pszInstanceName: LPWSTR,
    pub pszHostName: LPWSTR,
    pub ip4Address: *mut IP4_ADDRESS,
    pub ip6Address: *mut IP6_ADDRESS,
    pub wPort: WORD,
    pub wPriority: WORD,
    pub wWeight: WORD,
    pub dwPropertyCount: DWORD,
    pub keys: *mut PWSTR,
    pub values: *mut PWSTR,
    pub dwInterfaceIndex: DWORD,
}
#[test]
fn bindgen_test_layout__DNS_SERVICE_INSTANCE() {
    assert_eq!(
        ::std::mem::size_of::<_DNS_SERVICE_INSTANCE>(),
        72usize,
        concat!("Size of: ", stringify!(_DNS_SERVICE_INSTANCE))
    );
    assert_eq!(
        ::std::mem::align_of::<_DNS_SERVICE_INSTANCE>(),
        8usize,
        concat!("Alignment of ", stringify!(_DNS_SERVICE_INSTANCE))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).pszInstanceName as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(pszInstanceName)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).pszHostName as *const _ as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(pszHostName)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).ip4Address as *const _ as usize
        },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(ip4Address)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).ip6Address as *const _ as usize
        },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(ip6Address)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).wPort as *const _ as usize },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(wPort)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).wPriority as *const _ as usize },
        34usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(wPriority)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).wWeight as *const _ as usize },
        36usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(wWeight)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).dwPropertyCount as *const _ as usize
        },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(dwPropertyCount)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).keys as *const _ as usize },
        48usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(keys)
        )
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).values as *const _ as usize },
        56usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(values)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_INSTANCE>())).dwInterfaceIndex as *const _ as usize
        },
        64usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_INSTANCE),
            "::",
            stringify!(dwInterfaceIndex)
        )
    );
}
pub type PDNS_SERVICE_INSTANCE = *mut _DNS_SERVICE_INSTANCE;
extern "C" {
    pub fn DnsServiceConstructInstance(
        pServiceName: PCWSTR,
        pHostName: PCWSTR,
        pIp4: PIP4_ADDRESS,
        pIp6: PIP6_ADDRESS,
        wPort: WORD,
        wPriority: WORD,
        wWeight: WORD,
        dwPropertiesCount: DWORD,
        keys: *mut PCWSTR,
        values: *mut PCWSTR,
    ) -> PDNS_SERVICE_INSTANCE;
}
extern "C" {
    pub fn DnsServiceFreeInstance(pInstance: PDNS_SERVICE_INSTANCE);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DNS_SERVICE_CANCEL {
    pub reserved: PVOID,
}
#[test]
fn bindgen_test_layout__DNS_SERVICE_CANCEL() {
    assert_eq!(
        ::std::mem::size_of::<_DNS_SERVICE_CANCEL>(),
        8usize,
        concat!("Size of: ", stringify!(_DNS_SERVICE_CANCEL))
    );
    assert_eq!(
        ::std::mem::align_of::<_DNS_SERVICE_CANCEL>(),
        8usize,
        concat!("Alignment of ", stringify!(_DNS_SERVICE_CANCEL))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_DNS_SERVICE_CANCEL>())).reserved as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_CANCEL),
            "::",
            stringify!(reserved)
        )
    );
}
pub type PDNS_SERVICE_CANCEL = *mut _DNS_SERVICE_CANCEL;
pub type PDNS_SERVICE_REGISTER_COMPLETE =
    ::std::option::Option<unsafe extern "C" fn(DWORD, PVOID, PDNS_SERVICE_INSTANCE)>;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DNS_SERVICE_REGISTER_REQUEST {
    pub Version: ULONG,
    pub InterfaceIndex: ULONG,
    pub pServiceInstance: PDNS_SERVICE_INSTANCE,
    pub pRegisterCompletionCallback: PDNS_SERVICE_REGISTER_COMPLETE,
    pub pQueryContext: PVOID,
    pub hCredentials: HANDLE,
    pub unicastEnabled: BOOL,
}
#[test]
fn bindgen_test_layout__DNS_SERVICE_REGISTER_REQUEST() {
    assert_eq!(
        ::std::mem::size_of::<_DNS_SERVICE_REGISTER_REQUEST>(),
        48usize,
        concat!("Size of: ", stringify!(_DNS_SERVICE_REGISTER_REQUEST))
    );
    assert_eq!(
        ::std::mem::align_of::<_DNS_SERVICE_REGISTER_REQUEST>(),
        8usize,
        concat!("Alignment of ", stringify!(_DNS_SERVICE_REGISTER_REQUEST))
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).Version as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(Version)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).InterfaceIndex as *const _
                as usize
        },
        4usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(InterfaceIndex)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).pServiceInstance as *const _
                as usize
        },
        8usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(pServiceInstance)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).pRegisterCompletionCallback
                as *const _ as usize
        },
        16usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(pRegisterCompletionCallback)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).pQueryContext as *const _
                as usize
        },
        24usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(pQueryContext)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).hCredentials as *const _
                as usize
        },
        32usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(hCredentials)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_DNS_SERVICE_REGISTER_REQUEST>())).unicastEnabled as *const _
                as usize
        },
        40usize,
        concat!(
            "Offset of field: ",
            stringify!(_DNS_SERVICE_REGISTER_REQUEST),
            "::",
            stringify!(unicastEnabled)
        )
    );
}
pub type PDNS_SERVICE_REGISTER_REQUEST = *mut _DNS_SERVICE_REGISTER_REQUEST;
#[link(name = "dnsapi")]
extern "C" {
    pub fn DnsServiceRegisterCancel(pCancelHandle: PDNS_SERVICE_CANCEL) -> DWORD;
    pub fn DnsServiceRegister(
        pRequest: PDNS_SERVICE_REGISTER_REQUEST,
        pCancel: PDNS_SERVICE_CANCEL,
    ) -> DWORD;
    pub fn DnsServiceDeRegister(
        pRequest: PDNS_SERVICE_REGISTER_REQUEST,
        pCancel: PDNS_SERVICE_CANCEL,
    ) -> DWORD;
}