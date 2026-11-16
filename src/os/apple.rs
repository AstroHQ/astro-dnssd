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

/// Runs the body of an `extern "C"` callback, stopping a panic at the FFI boundary: unwinding into C aborts the
/// whole app, so a bug in one callback is logged and the event dropped instead
pub(crate) fn no_unwind<F: FnOnce()>(callback: &str, f: F) {
    if std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err() {
        log::error!(
            "Panic in {} callback, ignored to keep it from unwinding into C",
            callback
        );
    }
}
