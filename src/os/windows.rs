use std::os::windows::ffi::OsStrExt;

pub mod browse;
pub mod register;
pub fn to_utf16<S: AsRef<std::ffi::OsStr>>(s: S) -> Vec<u16> {
    s.as_ref()
        .encode_wide()
        .chain(Some(0u16).into_iter())
        .collect()
}
