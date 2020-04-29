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

fn dns_sd_include_path() -> Option<PathBuf> {
    if cfg_family_is("windows") {
        match var("BONJOUR_SDK_HOME") {
			Ok(path) =>
                Some(PathBuf::from(path).join("Include")),
			Err(e) => panic!("Can't find Bonjour SDK (download from https://developer.apple.com/bonjour/ ) at BONJOUR_SDK_HOME: {}", e),
		}
    } else {
        // TODO: apple & linux platforms
        None
    }
}

fn dns_sd_lib_path() -> Option<PathBuf> {
    if cfg_family_is("windows") {
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
    if cfg_family_is("windows") {
        println!("cargo:rustc-link-lib=dnssd");
    }
}

fn generate_bindings() {
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header_contents("wrapper.h", "#include <dns_sd.h>")
        .clang_arg(format!("-I{}", dns_sd_include_path().unwrap().display()))
        .whitelist_type("TXTRecord(.*)")
        // .whitelist_type("AOBoundedQueue(.*)")
        // .whitelist_type("AOCompressedTile(.*)")
        //
        // Any whitelisted function implicitly whitelists any types it uses.
        // .whitelist_function("AOEncoder(.*)")
        // .whitelist_function("AOBoundedQueue(.*)")
        // .whitelist_function("AOCompressedTile(.*)")
        // .whitelist_function("AOFrame(.*)")
        .whitelist_function("TXTRecord(.*)")
        .whitelist_type("DNSService(.*)")
        .whitelist_function("DNSService(.*)")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        // .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .whitelist_var("kDNSService(.*)")
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    generate_bindings();
    find_avahi_compat_dns_sd();
    find_windows_dns_sd();
}
