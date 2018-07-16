## A growable, reusable box for Rust.
 [![](https://travis-ci.org/mahou-shoujo/growable-rs.svg)](https://travis-ci.org/mahou-shoujo/growable-rs)
 
This crate provides a custom Box type with matching API that also allows to reuse the same
heap to store different types with the minimal amount of allocations and is supposed to be
used with a pool-based allocator such as the one also provided by this crate.

#### Notes
This crate uses a lot of ground-breaking features of Rust and therefore
is only available on the latest Nightly build.

#### Usage
At the current moment the crate is only available on GitHub.
```toml
[dependencies.growable]
git = "git://github.com/mahou-shoujo/growable-rs"
features = ["use_lifeguard"] # Optional built-in support for Lifeguard crate.
```
