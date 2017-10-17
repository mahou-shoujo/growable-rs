/*
 * Copyright (c) 2017 Eugene P. <mahou@shoujo.pw>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

extern crate growable;

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
fn assign_as_trait() {
    let buffer = Growable::new();
    let v = buffer.assign_as_trait::<_, Trait>(StandardType(24));
    assert_eq!(v.get(), 24);
    let buffer = v.free();
    let v = buffer.assign_as_trait::<_, Trait>(StandardType(48));
    assert_eq!(v.get(), 48);
}

#[test]
fn access_zst() {
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
fn drop() {
    use std::cell::Cell;
    use std::rc::Rc;
    let drop_counter = Rc::new(Cell::new(0));
    trait Droppable {}
    struct Foo(Rc<Cell<usize>>);
    impl Droppable for Foo {};
    impl Drop for Foo {
        fn drop(&mut self) { self.0.set(self.0.get() + 1); }
    }
    {
        let buffer = Growable::new();
        // Dropped by leaving the current scope:
        let _: Reusable<Droppable> = buffer.assign_as_trait(Foo(drop_counter.clone()));
        let buffer = Growable::new();
        let v: Reusable<Droppable> = buffer.assign_as_trait(Foo(drop_counter.clone()));
        // Dropped manually:
        v.free();
    }
    assert_eq!(drop_counter.get(), 2);
}