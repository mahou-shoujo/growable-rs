extern crate growable;

use std::mem::size_of;
use growable::*;

/// Some sample trait.
trait Trait {

    fn get(&self) -> u32;
}

/// Some basic trait implementor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StandardType(u32);

impl Trait for StandardType {

    fn get(&self) -> u32 {
        self.0
    }
}

/// Zero-sized trait implementor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ZST;

impl Trait for ZST {

    fn get(&self) -> u32 {
        42
    }
}

#[test]
fn access() {
    // --
    let buffer = Growable::new();
    let v: Reusable<[u8]> = buffer.consume([1u8, 2, 3, 4, 5, 6]);
    assert_eq!(  v.len(), 6);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6]);
    // --
    let buffer = v.free();
    let v: Reusable<[u8]> = buffer.consume([1u8, 2, 3, 4]);
    assert_eq!(  v.len(), 4);
    assert_eq!(&*v, &[1, 2, 3, 4]);
    // --
    let buffer = v.free();
    let v: Reusable<[u8]> = buffer.consume([1u8, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(  v.len(), 9);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn access_as_trait() {
    // --
    let buffer = Growable::new();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.alignment(), 1);
    let v: Reusable<Trait> = buffer.consume(StandardType(24));
    assert_eq!(v.get(), 24);
    // --
    let buffer = v.free();
    assert!( ! buffer.is_empty());
    assert_eq!(buffer.len(), size_of::<StandardType>());
    assert_eq!(buffer.alignment(), 4);
    let v: Reusable<Trait> = buffer.consume(StandardType(48));
    assert_eq!(v.get(), 48);
}

#[test]
fn access_zst() {
    // --
    let buffer = Growable::new();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn access_zst_as_trait() {
    // --
    let buffer = Growable::new();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = v.free();
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn drop() {
    // --
    use std::cell::Cell;
    use std::rc::Rc;
    // --
    let drop_counter = Rc::new(Cell::new(0));
    // --
    struct Foo(Rc<Cell<usize>>);
    impl Drop for Foo {
        fn drop(&mut self) { self.0.set(self.0.get() + 1); }
    }
    // --
    {
        let buffer = Growable::new();
        // Dropped by leaving the current scope:
        let _ = buffer.consume(Foo(Rc::clone(&drop_counter)));
        let buffer = Growable::new();
        let v = buffer.consume(Foo(Rc::clone(&drop_counter)));
        // Dropped manually:
        v.free();
    }
    assert_eq!(drop_counter.get(), 2);
}

#[test]
fn with_capacity() {
    // --
    let buffer = Growable::with_capacity(0, 1);
    assert!(   buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.alignment(), 1);
    // --
    let buffer = Growable::with_capacity(1, 2);
    assert!( ! buffer.is_empty());
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer.alignment(), 2);
}

#[test]
#[should_panic]
fn with_invalid_capacity_some_len_some_alignment() {
    let _ = Growable::with_capacity(8, 7);
}

#[test]
#[should_panic]
fn with_invalid_capacity_some_len_none_alignment() {
    let _ = Growable::with_capacity(4, 0);
}

#[test]
#[should_panic]
fn with_invalid_capacity_none_len_some_alignment() {
    let _ = Growable::with_capacity(0, 7);
}

#[test]
#[should_panic]
fn with_invalid_capacity_none_len_none_alignment() {
    let _ = Growable::with_capacity(0, 0);
}

#[test]
fn clone_growable() {
    let a = Growable::with_capacity_for_type::<usize>();
    let b = a.clone();
    let a = a.consume(3);
    let b = b.consume(7);
    assert_eq!(*a, 3);
    assert_eq!(*b, 7);
}

#[test]
fn clone_reusable() {
    let a = Growable::with_capacity_for_type::<usize>();
    let a = a.consume(4);
    let b = a.clone();
    assert_eq!(*a, 4);
    assert_eq!(*b, 4);
}
