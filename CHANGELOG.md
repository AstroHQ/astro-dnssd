# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -

## v0.3.10 - 2026-11-16
#### Bug Fixes
- free the register reply sender as its own type - (c0488af) - *jfro*
- no aborts, hangs or freed pointers in Apple dns_sd callbacks - (f65ff67) - *jfro*
- removed services carry the interface their added event had - (e8b25d7) - *jfro*
- deliver unresolved services instead of Timeout, allow recv_timeout(Duration::MAX) - (41839fd) - *jfro*
- never block or drop events in browse callbacks - (091a51b) - *jfro*

- - -
## v0.3.9 - 2026-11-09
#### Features
- ability to update TXT records of existing registration - (81ab2fa) - *jfro*

#### Bug Fixes
- **(windows)** hold de-register state until the callback succeeds - (62dc7f6) - *jfro*
- some deregistration edge cases - (17771cf) - *jfro*

- - -