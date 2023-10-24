## A growable, reusable box for Rust.
[![Build Status](https://github.com/mahou-shoujo/growable-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/mahou-shoujo/growable-rs/actions/workflows/rust.yml)
 
This crate provides a custom Box type with matching API that also allows to reuse the same
memory block to store different types with the minimal amount of allocations and is supposed to be
used with a pool-based allocator (such as the one provided by this crate).

#### Notes
The implementation depends on some unstable features:
1. [`allocator-api`](https://doc.rust-lang.org/unstable-book/library-features/allocator-api.html)
2. [`unsize`](https://doc.rust-lang.org/unstable-book/library-features/unsize.html)
3. [`coerce-unsized`](https://doc.rust-lang.org/unstable-book/library-features/coerce-unsized.html)
4. [`slice_ptr_get`](https://doc.rust-lang.org/unstable-book/library-features/slice-ptr-get.html)

Things can break randomly and the minimal supported version of rustc will be shifted accordingly.
Right now it is `rustc 1.75.0-nightly (1c05d50c8 2023-10-21)`.
