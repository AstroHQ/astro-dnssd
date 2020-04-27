# Astro DNS-SD (Rust Wrapper for Bonjour/DNS-SD APIs)

Minimal but friendly safe wrapper around dns-sd(Bonjour, mDNS, Zeroconf DNS) APIs.

## Features

### Complete

- Service registration
- TXTRecord support for service registration

### In Progress

- Service browsing

### Todo

- How to check for more (select() on socket, but has to be win32 friendly)
- Record creation
- Name resolution
- Port map
- Tests
- Documentation
- Pure Rust TXT code?
- Interior mutability? (Can we reduce the &mut arguments some?)

## Example

```rust
    let mut txt = TXTRecord::new();
    let _ = txt.insert("s", Some("open"));
    let mut service = DNSServiceBuilder::new("_rust._tcp")
        .with_port(2048)
        .with_name("MyRustService")
        .with_txt_record(txt)
        .build()
        .unwrap();
    let _result = service.register(|reply| match reply {
        Ok(reply) => println!("Successful reply: {:?}", reply),
        Err(e) => println!("Error registering: {:?}", e),
    });
    loop {
        service.process_result();
    }
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
