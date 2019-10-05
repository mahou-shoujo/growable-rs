use growable::*;
use std::mem::{align_of, size_of};

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
    assert_eq!(v.len(), 6);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6]);
    // --
    let buffer = Reusable::free(v);
    let v: Reusable<[u8]> = buffer.consume([1u8, 2, 3, 4]);
    assert_eq!(v.len(), 4);
    assert_eq!(&*v, &[1, 2, 3, 4]);
    // --
    let buffer = Reusable::free(v);
    let v: Reusable<[u8]> = buffer.consume([1u8, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(v.len(), 9);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn access_as_trait() {
    // --
    let buffer = Growable::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.alignment(), 1);
    let v: Reusable<dyn Trait> = buffer.consume(StandardType(24));
    assert_eq!(v.get(), 24);
    // --
    let buffer = Reusable::free(v);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), size_of::<StandardType>());
    assert_eq!(buffer.alignment(), align_of::<StandardType>());
    let v: Reusable<dyn Trait> = buffer.consume(StandardType(48));
    assert_eq!(v.get(), 48);
}

#[test]
fn access_zst() {
    // --
    let buffer = Growable::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn access_zst_as_trait() {
    // --
    let buffer = Growable::new();
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<dyn Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<dyn Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<dyn Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    let v: Reusable<dyn Trait> = buffer.consume(ZST);
    assert_eq!(v.get(), 42);
    // --
    let buffer = Reusable::free(v);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
}

#[test]
fn free_move() {
    struct Movable {
        string: String,
    }
    let buffer = Growable::new();
    let v = buffer.consume(Movable {
        string: String::from("Foo/Bar/Baz"),
    });
    let (v, buffer) = Reusable::free_move(v);
    assert_eq!(v.string.as_str(), "Foo/Bar/Baz");
    assert_eq!(buffer.len(), size_of::<Movable>());
    assert_eq!(buffer.alignment(), align_of::<Movable>());
}

#[test]
fn drop() {
    // --
    use std::{cell::Cell, rc::Rc};
    // --
    let drop_counter = Rc::new(Cell::new(0));
    // --
    struct Foo(Rc<Cell<usize>>);
    impl Drop for Foo {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    // --
    {
        let buffer = Growable::new();
        // Dropped by leaving the current scope:
        let _ = buffer.consume(Foo(Rc::clone(&drop_counter)));
        let buffer = Growable::new();
        let v = buffer.consume(Foo(Rc::clone(&drop_counter)));
        // Dropped manually:
        Reusable::free(v);
    }
    assert_eq!(drop_counter.get(), 2);
}

#[test]
fn with_capacity() {
    // --
    let buffer = Growable::with_capacity(0, 1);
    assert!(buffer.is_empty());
    assert_eq!(buffer.len(), 0);
    assert_eq!(buffer.alignment(), 1);
    // --
    let buffer = Growable::with_capacity(1, 1);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer.alignment(), 1);
    // --
    let buffer = Growable::with_capacity(2, 1);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer.alignment(), 1);
    // --
    let buffer = Growable::with_capacity(2, 2);
    assert!(!buffer.is_empty());
    assert_eq!(buffer.len(), 2);
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

#[test]
fn pool() {
    let mut pool = GrowablePoolBuilder::default()
        .with_default_capacity(128)
        .with_default_ptr_alignment(16)
        .with_capacity(2)
        .enable_overgrow(true)
        .build();
    assert_eq!(pool.len(), 2);
    let a = pool.allocate(1);
    assert_eq!(pool.len(), 1);
    let b = pool.allocate(2);
    assert_eq!(pool.len(), 0);
    pool.free(b);
    assert_eq!(pool.len(), 1);
    pool.free(a);
    assert_eq!(pool.len(), 2);
}

#[test]
fn pool_on_demand() {
    let mut pool = GrowablePoolBuilder::default()
        .with_default_capacity(128)
        .with_default_ptr_alignment(16)
        .with_capacity(0)
        .enable_overgrow(false)
        .build();
    assert_eq!(pool.len(), 0);
    let a = pool.allocate(1);
    assert_eq!(pool.len(), 0);
    let b = pool.allocate(2);
    assert_eq!(pool.len(), 0);
    pool.free(b);
    assert_eq!(pool.len(), 0);
    pool.free(a);
    assert_eq!(pool.len(), 0);
    let mut pool = GrowablePoolBuilder::default()
        .with_default_capacity(128)
        .with_default_ptr_alignment(16)
        .with_capacity(0)
        .enable_overgrow(true)
        .build();
    assert_eq!(pool.len(), 0);
    let a = pool.allocate(1);
    assert_eq!(pool.len(), 0);
    let b = pool.allocate(2);
    assert_eq!(pool.len(), 0);
    pool.free(b);
    assert_eq!(pool.len(), 1);
    pool.free(a);
    assert_eq!(pool.len(), 2);
}
