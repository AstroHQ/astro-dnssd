#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code
)]

pub const kDNSServiceMaxServiceName: u32 = 64;
pub const kDNSServiceMaxDomainName: u32 = 1009;
pub const kDNSServiceInterfaceIndexAny: u32 = 0;
pub const kDNSServiceProperty_DaemonVersion: &'static [u8; 14usize] = b"DaemonVersion\0";
pub type dnssd_sock_t = ::std::os::raw::c_int;
pub type dispatch_queue_t = *mut dispatch_queue_s;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct dispatch_queue_s {
    pub _address: u8,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DNSServiceRef_t {
    _unused: [u8; 0],
}
pub type DNSServiceRef = *mut _DNSServiceRef_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _DNSRecordRef_t {
    _unused: [u8; 0],
}
pub type DNSRecordRef = *mut _DNSRecordRef_t;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct sockaddr {
    _unused: [u8; 0],
}
pub const kDNSServiceFlagsMoreComing: _bindgen_ty_3 = 1;
pub const kDNSServiceFlagsAutoTrigger: _bindgen_ty_3 = 1;
pub const kDNSServiceFlagsAdd: _bindgen_ty_3 = 2;
pub const kDNSServiceFlagsDefault: _bindgen_ty_3 = 4;
pub const kDNSServiceFlagsNoAutoRename: _bindgen_ty_3 = 8;
pub const kDNSServiceFlagsShared: _bindgen_ty_3 = 16;
pub const kDNSServiceFlagsUnique: _bindgen_ty_3 = 32;
pub const kDNSServiceFlagsBrowseDomains: _bindgen_ty_3 = 64;
pub const kDNSServiceFlagsRegistrationDomains: _bindgen_ty_3 = 128;
pub const kDNSServiceFlagsLongLivedQuery: _bindgen_ty_3 = 256;
pub const kDNSServiceFlagsAllowRemoteQuery: _bindgen_ty_3 = 512;
pub const kDNSServiceFlagsForceMulticast: _bindgen_ty_3 = 1024;
pub const kDNSServiceFlagsForce: _bindgen_ty_3 = 2048;
pub const kDNSServiceFlagsKnownUnique: _bindgen_ty_3 = 2048;
pub const kDNSServiceFlagsReturnIntermediates: _bindgen_ty_3 = 4096;
pub const kDNSServiceFlagsShareConnection: _bindgen_ty_3 = 16384;
pub const kDNSServiceFlagsSuppressUnusable: _bindgen_ty_3 = 32768;
pub const kDNSServiceFlagsTimeout: _bindgen_ty_3 = 65536;
pub const kDNSServiceFlagsIncludeP2P: _bindgen_ty_3 = 131072;
pub const kDNSServiceFlagsWakeOnResolve: _bindgen_ty_3 = 262144;
pub const kDNSServiceFlagsBackgroundTrafficClass: _bindgen_ty_3 = 524288;
pub const kDNSServiceFlagsIncludeAWDL: _bindgen_ty_3 = 1048576;
pub const kDNSServiceFlagsValidate: _bindgen_ty_3 = 2097152;
pub const kDNSServiceFlagsSecure: _bindgen_ty_3 = 2097168;
pub const kDNSServiceFlagsInsecure: _bindgen_ty_3 = 2097184;
pub const kDNSServiceFlagsBogus: _bindgen_ty_3 = 2097216;
pub const kDNSServiceFlagsIndeterminate: _bindgen_ty_3 = 2097280;
pub const kDNSServiceFlagsUnicastResponse: _bindgen_ty_3 = 4194304;
pub const kDNSServiceFlagsValidateOptional: _bindgen_ty_3 = 8388608;
pub const kDNSServiceFlagsWakeOnlyService: _bindgen_ty_3 = 16777216;
pub const kDNSServiceFlagsThresholdOne: _bindgen_ty_3 = 33554432;
pub const kDNSServiceFlagsThresholdFinder: _bindgen_ty_3 = 67108864;
pub const kDNSServiceFlagsThresholdReached: _bindgen_ty_3 = 33554432;
pub const kDNSServiceFlagsPrivateOne: _bindgen_ty_3 = 8192;
pub const kDNSServiceFlagsPrivateTwo: _bindgen_ty_3 = 134217728;
pub const kDNSServiceFlagsPrivateThree: _bindgen_ty_3 = 268435456;
pub const kDNSServiceFlagsPrivateFour: _bindgen_ty_3 = 536870912;
pub const kDNSServiceFlagsPrivateFive: _bindgen_ty_3 = 1073741824;
pub const kDNSServiceFlagAnsweredFromCache: _bindgen_ty_3 = 1073741824;
pub const kDNSServiceFlagsAllowExpiredAnswers: _bindgen_ty_3 = 2147483648;
pub const kDNSServiceFlagsExpiredAnswer: _bindgen_ty_3 = 2147483648;
pub type _bindgen_ty_3 = u32;
pub const kDNSServiceProtocol_IPv4: _bindgen_ty_4 = 1;
pub const kDNSServiceProtocol_IPv6: _bindgen_ty_4 = 2;
pub const kDNSServiceProtocol_UDP: _bindgen_ty_4 = 16;
pub const kDNSServiceProtocol_TCP: _bindgen_ty_4 = 32;
pub type _bindgen_ty_4 = u32;
pub const kDNSServiceClass_IN: _bindgen_ty_5 = 1;
pub type _bindgen_ty_5 = u32;
pub const kDNSServiceType_A: _bindgen_ty_6 = 1;
pub const kDNSServiceType_NS: _bindgen_ty_6 = 2;
pub const kDNSServiceType_MD: _bindgen_ty_6 = 3;
pub const kDNSServiceType_MF: _bindgen_ty_6 = 4;
pub const kDNSServiceType_CNAME: _bindgen_ty_6 = 5;
pub const kDNSServiceType_SOA: _bindgen_ty_6 = 6;
pub const kDNSServiceType_MB: _bindgen_ty_6 = 7;
pub const kDNSServiceType_MG: _bindgen_ty_6 = 8;
pub const kDNSServiceType_MR: _bindgen_ty_6 = 9;
pub const kDNSServiceType_NULL: _bindgen_ty_6 = 10;
pub const kDNSServiceType_WKS: _bindgen_ty_6 = 11;
pub const kDNSServiceType_PTR: _bindgen_ty_6 = 12;
pub const kDNSServiceType_HINFO: _bindgen_ty_6 = 13;
pub const kDNSServiceType_MINFO: _bindgen_ty_6 = 14;
pub const kDNSServiceType_MX: _bindgen_ty_6 = 15;
pub const kDNSServiceType_TXT: _bindgen_ty_6 = 16;
pub const kDNSServiceType_RP: _bindgen_ty_6 = 17;
pub const kDNSServiceType_AFSDB: _bindgen_ty_6 = 18;
pub const kDNSServiceType_X25: _bindgen_ty_6 = 19;
pub const kDNSServiceType_ISDN: _bindgen_ty_6 = 20;
pub const kDNSServiceType_RT: _bindgen_ty_6 = 21;
pub const kDNSServiceType_NSAP: _bindgen_ty_6 = 22;
pub const kDNSServiceType_NSAP_PTR: _bindgen_ty_6 = 23;
pub const kDNSServiceType_SIG: _bindgen_ty_6 = 24;
pub const kDNSServiceType_KEY: _bindgen_ty_6 = 25;
pub const kDNSServiceType_PX: _bindgen_ty_6 = 26;
pub const kDNSServiceType_GPOS: _bindgen_ty_6 = 27;
pub const kDNSServiceType_AAAA: _bindgen_ty_6 = 28;
pub const kDNSServiceType_LOC: _bindgen_ty_6 = 29;
pub const kDNSServiceType_NXT: _bindgen_ty_6 = 30;
pub const kDNSServiceType_EID: _bindgen_ty_6 = 31;
pub const kDNSServiceType_NIMLOC: _bindgen_ty_6 = 32;
pub const kDNSServiceType_SRV: _bindgen_ty_6 = 33;
pub const kDNSServiceType_ATMA: _bindgen_ty_6 = 34;
pub const kDNSServiceType_NAPTR: _bindgen_ty_6 = 35;
pub const kDNSServiceType_KX: _bindgen_ty_6 = 36;
pub const kDNSServiceType_CERT: _bindgen_ty_6 = 37;
pub const kDNSServiceType_A6: _bindgen_ty_6 = 38;
pub const kDNSServiceType_DNAME: _bindgen_ty_6 = 39;
pub const kDNSServiceType_SINK: _bindgen_ty_6 = 40;
pub const kDNSServiceType_OPT: _bindgen_ty_6 = 41;
pub const kDNSServiceType_APL: _bindgen_ty_6 = 42;
pub const kDNSServiceType_DS: _bindgen_ty_6 = 43;
pub const kDNSServiceType_SSHFP: _bindgen_ty_6 = 44;
pub const kDNSServiceType_IPSECKEY: _bindgen_ty_6 = 45;
pub const kDNSServiceType_RRSIG: _bindgen_ty_6 = 46;
pub const kDNSServiceType_NSEC: _bindgen_ty_6 = 47;
pub const kDNSServiceType_DNSKEY: _bindgen_ty_6 = 48;
pub const kDNSServiceType_DHCID: _bindgen_ty_6 = 49;
pub const kDNSServiceType_NSEC3: _bindgen_ty_6 = 50;
pub const kDNSServiceType_NSEC3PARAM: _bindgen_ty_6 = 51;
pub const kDNSServiceType_HIP: _bindgen_ty_6 = 55;
pub const kDNSServiceType_SPF: _bindgen_ty_6 = 99;
pub const kDNSServiceType_UINFO: _bindgen_ty_6 = 100;
pub const kDNSServiceType_UID: _bindgen_ty_6 = 101;
pub const kDNSServiceType_GID: _bindgen_ty_6 = 102;
pub const kDNSServiceType_UNSPEC: _bindgen_ty_6 = 103;
pub const kDNSServiceType_TKEY: _bindgen_ty_6 = 249;
pub const kDNSServiceType_TSIG: _bindgen_ty_6 = 250;
pub const kDNSServiceType_IXFR: _bindgen_ty_6 = 251;
pub const kDNSServiceType_AXFR: _bindgen_ty_6 = 252;
pub const kDNSServiceType_MAILB: _bindgen_ty_6 = 253;
pub const kDNSServiceType_MAILA: _bindgen_ty_6 = 254;
pub const kDNSServiceType_ANY: _bindgen_ty_6 = 255;
pub type _bindgen_ty_6 = u32;
pub const kDNSServiceErr_NoError: _bindgen_ty_7 = 0;
pub const kDNSServiceErr_Unknown: _bindgen_ty_7 = -65537;
pub const kDNSServiceErr_NoSuchName: _bindgen_ty_7 = -65538;
pub const kDNSServiceErr_NoMemory: _bindgen_ty_7 = -65539;
pub const kDNSServiceErr_BadParam: _bindgen_ty_7 = -65540;
pub const kDNSServiceErr_BadReference: _bindgen_ty_7 = -65541;
pub const kDNSServiceErr_BadState: _bindgen_ty_7 = -65542;
pub const kDNSServiceErr_BadFlags: _bindgen_ty_7 = -65543;
pub const kDNSServiceErr_Unsupported: _bindgen_ty_7 = -65544;
pub const kDNSServiceErr_NotInitialized: _bindgen_ty_7 = -65545;
pub const kDNSServiceErr_AlreadyRegistered: _bindgen_ty_7 = -65547;
pub const kDNSServiceErr_NameConflict: _bindgen_ty_7 = -65548;
pub const kDNSServiceErr_Invalid: _bindgen_ty_7 = -65549;
pub const kDNSServiceErr_Firewall: _bindgen_ty_7 = -65550;
pub const kDNSServiceErr_Incompatible: _bindgen_ty_7 = -65551;
pub const kDNSServiceErr_BadInterfaceIndex: _bindgen_ty_7 = -65552;
pub const kDNSServiceErr_Refused: _bindgen_ty_7 = -65553;
pub const kDNSServiceErr_NoSuchRecord: _bindgen_ty_7 = -65554;
pub const kDNSServiceErr_NoAuth: _bindgen_ty_7 = -65555;
pub const kDNSServiceErr_NoSuchKey: _bindgen_ty_7 = -65556;
pub const kDNSServiceErr_NATTraversal: _bindgen_ty_7 = -65557;
pub const kDNSServiceErr_DoubleNAT: _bindgen_ty_7 = -65558;
pub const kDNSServiceErr_BadTime: _bindgen_ty_7 = -65559;
pub const kDNSServiceErr_BadSig: _bindgen_ty_7 = -65560;
pub const kDNSServiceErr_BadKey: _bindgen_ty_7 = -65561;
pub const kDNSServiceErr_Transient: _bindgen_ty_7 = -65562;
pub const kDNSServiceErr_ServiceNotRunning: _bindgen_ty_7 = -65563;
pub const kDNSServiceErr_NATPortMappingUnsupported: _bindgen_ty_7 = -65564;
pub const kDNSServiceErr_NATPortMappingDisabled: _bindgen_ty_7 = -65565;
pub const kDNSServiceErr_NoRouter: _bindgen_ty_7 = -65566;
pub const kDNSServiceErr_PollingMode: _bindgen_ty_7 = -65567;
pub const kDNSServiceErr_Timeout: _bindgen_ty_7 = -65568;
pub const kDNSServiceErr_DefunctConnection: _bindgen_ty_7 = -65569;
pub type _bindgen_ty_7 = i32;
pub type DNSServiceFlags = u32;
pub type DNSServiceProtocol = u32;
pub type DNSServiceErrorType = i32;
extern "C" {
    pub fn DNSServiceGetProperty(
        property: *const ::std::os::raw::c_char,
        result: *mut ::std::os::raw::c_void,
        size: *mut u32,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceRefSockFD(sdRef: DNSServiceRef) -> dnssd_sock_t;
}
extern "C" {
    pub fn DNSServiceProcessResult(sdRef: DNSServiceRef) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceRefDeallocate(sdRef: DNSServiceRef);
}
pub type DNSServiceDomainEnumReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        replyDomain: *const ::std::os::raw::c_char,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceEnumerateDomains(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        callBack: DNSServiceDomainEnumReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceRegisterReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        errorCode: DNSServiceErrorType,
        name: *const ::std::os::raw::c_char,
        regtype: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceRegister(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        name: *const ::std::os::raw::c_char,
        regtype: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
        host: *const ::std::os::raw::c_char,
        port: u16,
        txtLen: u16,
        txtRecord: *const ::std::os::raw::c_void,
        callBack: DNSServiceRegisterReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceAddRecord(
        sdRef: DNSServiceRef,
        RecordRef: *mut DNSRecordRef,
        flags: DNSServiceFlags,
        rrtype: u16,
        rdlen: u16,
        rdata: *const ::std::os::raw::c_void,
        ttl: u32,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceUpdateRecord(
        sdRef: DNSServiceRef,
        RecordRef: DNSRecordRef,
        flags: DNSServiceFlags,
        rdlen: u16,
        rdata: *const ::std::os::raw::c_void,
        ttl: u32,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceRemoveRecord(
        sdRef: DNSServiceRef,
        RecordRef: DNSRecordRef,
        flags: DNSServiceFlags,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceBrowseReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        serviceName: *const ::std::os::raw::c_char,
        regtype: *const ::std::os::raw::c_char,
        replyDomain: *const ::std::os::raw::c_char,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceBrowse(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        regtype: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
        callBack: DNSServiceBrowseReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceResolveReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        fullname: *const ::std::os::raw::c_char,
        hosttarget: *const ::std::os::raw::c_char,
        port: u16,
        txtLen: u16,
        txtRecord: *const ::std::os::raw::c_uchar,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceResolve(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        name: *const ::std::os::raw::c_char,
        regtype: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
        callBack: DNSServiceResolveReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceQueryRecordReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        fullname: *const ::std::os::raw::c_char,
        rrtype: u16,
        rrclass: u16,
        rdlen: u16,
        rdata: *const ::std::os::raw::c_void,
        ttl: u32,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceQueryRecord(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        fullname: *const ::std::os::raw::c_char,
        rrtype: u16,
        rrclass: u16,
        callBack: DNSServiceQueryRecordReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceGetAddrInfoReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        hostname: *const ::std::os::raw::c_char,
        address: *const sockaddr,
        ttl: u32,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceGetAddrInfo(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        protocol: DNSServiceProtocol,
        hostname: *const ::std::os::raw::c_char,
        callBack: DNSServiceGetAddrInfoReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceCreateConnection(sdRef: *mut DNSServiceRef) -> DNSServiceErrorType;
}
pub type DNSServiceRegisterRecordReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        RecordRef: DNSRecordRef,
        flags: DNSServiceFlags,
        errorCode: DNSServiceErrorType,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceRegisterRecord(
        sdRef: DNSServiceRef,
        RecordRef: *mut DNSRecordRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        fullname: *const ::std::os::raw::c_char,
        rrtype: u16,
        rrclass: u16,
        rdlen: u16,
        rdata: *const ::std::os::raw::c_void,
        ttl: u32,
        callBack: DNSServiceRegisterRecordReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceReconfirmRecord(
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        fullname: *const ::std::os::raw::c_char,
        rrtype: u16,
        rrclass: u16,
        rdlen: u16,
        rdata: *const ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceNATPortMappingReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        errorCode: DNSServiceErrorType,
        externalAddress: u32,
        protocol: DNSServiceProtocol,
        internalPort: u16,
        externalPort: u16,
        ttl: u32,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceNATPortMappingCreate(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        interfaceIndex: u32,
        protocol: DNSServiceProtocol,
        internalPort: u16,
        externalPort: u16,
        ttl: u32,
        callBack: DNSServiceNATPortMappingReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceConstructFullName(
        fullName: *mut ::std::os::raw::c_char,
        service: *const ::std::os::raw::c_char,
        regtype: *const ::std::os::raw::c_char,
        domain: *const ::std::os::raw::c_char,
    ) -> DNSServiceErrorType;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union _TXTRecordRef_t {
    pub PrivateData: [::std::os::raw::c_char; 16usize],
    pub ForceNaturalAlignment: *mut ::std::os::raw::c_char,
    _bindgen_union_align: [u64; 2usize],
}
#[test]
fn bindgen_test_layout__TXTRecordRef_t() {
    assert_eq!(
        ::std::mem::size_of::<_TXTRecordRef_t>(),
        16usize,
        concat!("Size of: ", stringify!(_TXTRecordRef_t))
    );
    assert_eq!(
        ::std::mem::align_of::<_TXTRecordRef_t>(),
        8usize,
        concat!("Alignment of ", stringify!(_TXTRecordRef_t))
    );
    assert_eq!(
        unsafe { &(*(::std::ptr::null::<_TXTRecordRef_t>())).PrivateData as *const _ as usize },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_TXTRecordRef_t),
            "::",
            stringify!(PrivateData)
        )
    );
    assert_eq!(
        unsafe {
            &(*(::std::ptr::null::<_TXTRecordRef_t>())).ForceNaturalAlignment as *const _ as usize
        },
        0usize,
        concat!(
            "Offset of field: ",
            stringify!(_TXTRecordRef_t),
            "::",
            stringify!(ForceNaturalAlignment)
        )
    );
}
pub type TXTRecordRef = _TXTRecordRef_t;
extern "C" {
    pub fn TXTRecordCreate(
        txtRecord: *mut TXTRecordRef,
        bufferLen: u16,
        buffer: *mut ::std::os::raw::c_void,
    );
}
extern "C" {
    pub fn TXTRecordDeallocate(txtRecord: *mut TXTRecordRef);
}
extern "C" {
    pub fn TXTRecordSetValue(
        txtRecord: *mut TXTRecordRef,
        key: *const ::std::os::raw::c_char,
        valueSize: u8,
        value: *const ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn TXTRecordRemoveValue(
        txtRecord: *mut TXTRecordRef,
        key: *const ::std::os::raw::c_char,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn TXTRecordGetLength(txtRecord: *const TXTRecordRef) -> u16;
}
extern "C" {
    pub fn TXTRecordGetBytesPtr(txtRecord: *const TXTRecordRef) -> *const ::std::os::raw::c_void;
}
extern "C" {
    pub fn TXTRecordContainsKey(
        txtLen: u16,
        txtRecord: *const ::std::os::raw::c_void,
        key: *const ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn TXTRecordGetValuePtr(
        txtLen: u16,
        txtRecord: *const ::std::os::raw::c_void,
        key: *const ::std::os::raw::c_char,
        valueLen: *mut u8,
    ) -> *const ::std::os::raw::c_void;
}
extern "C" {
    pub fn TXTRecordGetCount(txtLen: u16, txtRecord: *const ::std::os::raw::c_void) -> u16;
}
extern "C" {
    pub fn TXTRecordGetItemAtIndex(
        txtLen: u16,
        txtRecord: *const ::std::os::raw::c_void,
        itemIndex: u16,
        keyBufLen: u16,
        key: *mut ::std::os::raw::c_char,
        valueLen: *mut u8,
        value: *mut *const ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
extern "C" {
    pub fn DNSServiceSetDispatchQueue(
        service: DNSServiceRef,
        queue: dispatch_queue_t,
    ) -> DNSServiceErrorType;
}
pub type DNSServiceSleepKeepaliveReply = ::std::option::Option<
    unsafe extern "C" fn(
        sdRef: DNSServiceRef,
        errorCode: DNSServiceErrorType,
        context: *mut ::std::os::raw::c_void,
    ),
>;
extern "C" {
    pub fn DNSServiceSleepKeepalive(
        sdRef: *mut DNSServiceRef,
        flags: DNSServiceFlags,
        fd: ::std::os::raw::c_int,
        timeout: ::std::os::raw::c_uint,
        callBack: DNSServiceSleepKeepaliveReply,
        context: *mut ::std::os::raw::c_void,
    ) -> DNSServiceErrorType;
}
