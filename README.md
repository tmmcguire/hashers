# Hashers

This crate is a collection of hashing-related functionality, for use
with Rust's `std::collections::HashMap`, `HashSet`, and so forth.

## Disclaimer

**None** of this is *cryptographically secure*. Attempting to use this
for cryptographic purposes is not recommended. I am not a cryptographer;
I don't even play one on TV.

Many (most? all?) of these functions are not designed to prevent
collision-based denial of service attacks. Rust's default hash function
is SipHash (1-3?), which is designed for that. Many of these functions
are intended to be used for performance purposes where that form of
security is not needed.
