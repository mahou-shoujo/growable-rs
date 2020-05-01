## A growable, reusable box for Rust.
[![Build Status](https://travis-ci.com/mahou-shoujo/growable-rs.svg?branch=master)](https://travis-ci.com/mahou-shoujo/growable-rs)
 
This crate provides a custom Box type with matching API that also allows to reuse the same
memory block to store different types with the minimal amount of allocations and is supposed to be
used with a pool-based allocator (such as the one provided by this crate).

#### Notes
The implementation depends on some unstable features:
1. [`allocator-api`](https://doc.rust-lang.org/unstable-book/library-features/allocator-api.html)
2. [`unsize`](https://doc.rust-lang.org/unstable-book/library-features/unsize.html)
3. [`coerce-unsized`](https://doc.rust-lang.org/unstable-book/library-features/coerce-unsized.html)

Things can break randomly and the minimal supported version of rustc will be shifted accordingly.
Right now it is `rustc 1.45.0-nightly (7ced01a73 2020-04-30)`.
