use std::env::{var, var_os};
use std::path::PathBuf;

fn cfg_arch() -> String {
    var("CARGO_CFG_TARGET_ARCH").expect("couldn't find target architecture")
}

fn cfg_family_is(family: &str) -> bool {
    var_os("CARGO_CFG_TARGET_FAMILY").unwrap() == *family
}

fn cfg_os_is(family: &str) -> bool {
    var_os("CARGO_CFG_TARGET_OS").unwrap() == *family
}

fn find_avahi_compat_dns_sd() {
    // on unix but not darwin link avahi compat
    if cfg_family_is("unix") && !(cfg_os_is("macos") || cfg_os_is("ios")) {
        pkg_config::probe_library("avahi-compat-libdns_sd").unwrap();
    }
}

fn dns_sd_lib_path() -> Option<PathBuf> {
    if cfg!(feature = "win-bonjour") && cfg_family_is("windows") {
        let platform = match cfg_arch().as_str() {
            "x86_64" => "x64",
            "x86" => "Win32",
            arch => panic!("unsupported target architecture: {:?}", arch),
        };
        match var("BONJOUR_SDK_HOME") {
            Ok(path) => Some(PathBuf::from(path).join("Lib").join(platform)),
            Err(e) => panic!("Can't find Bonjour SDK (download from https://developer.apple.com/bonjour/ ) at BONJOUR_SDK_HOME: {}", e),
        }
    } else {
        None
    }
}

fn find_windows_dns_sd() {
    if let Some(path) = dns_sd_lib_path() {
        println!("cargo:rustc-link-search=native={}", path.display())
    }
    if cfg!(feature = "win-bonjour") && cfg_family_is("windows") {
        println!("cargo:rustc-link-lib=dnssd");
    } else if cfg_family_is("windows") {
        println!("cargo:rustc-link-lib=dnsapi");
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    find_avahi_compat_dns_sd();
    find_windows_dns_sd();
}
