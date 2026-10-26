pub mod browse;
pub mod register;
mod txt;

/// Flags to OR in when a builder opts into peer-to-peer (AWDL) discovery/advertising
#[cfg(target_vendor = "apple")]
pub(crate) fn peer_to_peer_flags(enabled: bool) -> crate::ffi::apple::DNSServiceFlags {
    use crate::ffi::apple::{kDNSServiceFlagsIncludeAWDL, kDNSServiceFlagsIncludeP2P};
    if enabled {
        (kDNSServiceFlagsIncludeP2P | kDNSServiceFlagsIncludeAWDL) as _
    } else {
        0
    }
}
