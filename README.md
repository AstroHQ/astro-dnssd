# Astro DNS-SD (Rust Wrapper for Bonjour/DNS-SD APIs)

My attempt to learn wrapping C APIs in Rust, and aim for a minimal but friendly safe wrapper around dns-sd(Bonjour, mDNS, Zeroconf DNS) APIs.

## Status

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
