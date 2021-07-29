#[cfg(any(feature = "win-bonjour", not(target_os = "windows")))]
pub(crate) mod apple;
#[cfg(windows)]
pub(crate) mod windows;
