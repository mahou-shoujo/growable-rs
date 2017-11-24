## A growable, reusable box for Rust.
 [![](https://travis-ci.org/mahou-shoujo/growable-rs.svg)](https://travis-ci.org/mahou-shoujo/growable-rs)
 
This crate provides a custom Box type with matching API that also allows to reuse the same
memory chunk to store different types with minimal amount of allocations and is supposed to be
used with a custom, pool-based allocator of user's choice such as [Lifeguard](https://crates.io/crates/lifeguard).
#### Usage
At the current moment the crate is only available on GitHub.
```toml
[dependencies.growable]
git = "git://github.com/mahou-shoujo/growable-rs"
features = ["use_lifeguard"] # Optional built-in support for Lifeguard crate.
```
#### Nightly only!
Note that this crate uses a lot of ground-breaking features of Rust and therefore
is only maintained for the latest Nightly build.