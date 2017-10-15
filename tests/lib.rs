extern crate growable;

use growable::*;

trait Trait {

    fn get(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StandardType(usize);

impl Trait for StandardType {

    fn get(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ZST;

impl Trait for ZST {

    fn get(&self) -> usize {
        42
    }
}

#[test]
pub fn access() {
    let buffer = Growable::new();
    let v = buffer.assign_as_trait::<_, [u8]>([1, 2, 3, 4, 5, 6]);
    assert_eq!(v.len(), 6);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6]);
    let buffer = v.free();
    let v = buffer.assign_as_trait::<_, [u8]>([1, 2, 3, 4]);
    assert_eq!(v.len(), 4);
    assert_eq!(&*v, &[1, 2, 3, 4]);
    let buffer = v.free();
    let v = buffer.assign_as_trait::<_, [u8]>([1, 2, 3, 4, 5, 6, 7, 8, 9]);
    assert_eq!(v.len(), 9);
    assert_eq!(&*v, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
pub fn assign_as_trait() {
    let buffer = Growable::new();
    let v = buffer.assign_as_trait::<_, Trait>(StandardType(24));
    assert_eq!(v.get(), 24);
    let buffer = v.free();
    let v = buffer.assign_as_trait::<_, Trait>(StandardType(48));
    assert_eq!(v.get(), 48);
}

#[test]
pub fn access_zst() {
    let buffer = Growable::new();
    let v: Reusable<Trait> = buffer.assign_as_trait(ZST);
    assert_eq!(v.get(), 42);
    let buffer = v.free();
    let v: Reusable<Trait> = buffer.assign_as_trait(ZST);
    assert_eq!(v.get(), 42);
    let buffer = v.free();
    let v: Reusable<ZST> = buffer.assign(ZST);
    assert_eq!(v.get(), 42);
    let buffer = v.free();
    let v: Reusable<ZST> = buffer.assign(ZST);
    assert_eq!(v.get(), 42);
}

#[test]
pub fn drop() {
    let counter = ::std::rc::Rc::new(::std::cell::Cell::new(0usize));
    trait Test {}
    struct Foo(::std::rc::Rc<::std::cell::Cell<usize>>);
    impl Test for Foo {};
    impl Drop for Foo {
        fn drop(&mut self) { self.0.set(self.0.get() + 1); }
    }
    {
        let buffer = Growable::new();
        let _: Reusable<Test> = buffer.assign_as_trait(Foo(counter.to_owned()));
        let buffer = Growable::new();
        let v: Reusable<Test> = buffer.assign_as_trait(Foo(counter.to_owned()));
        v.free();
    }
    assert_eq!(counter.as_ref().get(), 2);
}