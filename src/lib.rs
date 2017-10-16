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

//! A growable, reusable box for Rust.
//!
//! This crate provides a custom Box type with matching API that also allows to reuse the same
//! box to store different types with minimal amount of allocations and is supposed to be
//! used with a custom, pool-based allocator of user's choice such as
//! [Lifeguard](https://crates.io/crates/lifeguard), but also can be used as a standalone library.
//!
//! Note that this crate uses a lot of ground-breaking features of Rust and therefore
//! is only available on current Nighty build.

#![warn(missing_docs)]
#![feature(allocator_api, alloc, coerce_unsized, unsize, unique)]

#[cfg(feature="use_lifeguard")] extern crate lifeguard;
extern crate alloc;

use std::marker::Unsize;
use std::ops;
use std::ops::CoerceUnsized;
use std::cmp;
use std::fmt;
use std::mem;
use std::ptr;
use std::ptr::Unique;
use alloc::allocator::{Alloc, Layout};
use alloc::heap::Heap;

/// A chunk of heap memory that can be assigned to a struct or a trait object.
/// Until assigned to some data it behaves similarly to a Box<[u8; N]>,
/// it can be cloned and would be dropped if leaves the scope.
///
/// # Examples
///
/// First, let's spawn a new Growable. In this case
/// no allocation would be performed on init.
///
/// ```
/// # use growable::*;
///   let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.assign(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
/// # let growable = arr.free();
/// # let arr: Reusable<[char; 6]> = growable.assign(['f', 'o', 'o', 'b', 'a', 'r']);
/// # assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// Now we can assign some data to it.
///
/// ```
/// # use growable::*;
/// # let growable = Growable::new();
///   let arr: Reusable<[char; 3]> = growable.assign(['f', 'o', 'o']);
///   assert_eq!(&*arr, &['f', 'o', 'o']);
/// # let growable = arr.free();
/// # let arr: Reusable<[char; 6]> = growable.assign(['f', 'o', 'o', 'b', 'a', 'r']);
/// # assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// Unwanted data could be then freed on demand, fetching Growable back.
/// Then it could be assigned to some data again and so on.
///
/// ```
/// # use growable::*;
/// # let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.assign(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
///   let growable = arr.free();
///   let arr: Reusable<[char; 6]> = growable.assign(['f', 'o', 'o', 'b', 'a', 'r']);
///   assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
pub enum Growable {
    /// Pre-allocated chunk of memory.
    Some {
        /// Memory block length.
        len: usize,
        /// Required alignment for the pointer.
        ptr_alignment: usize,
        /// Pointer.
        ptr: Unique<u8>
    },
    /// No assigned memory.
    None
}

#[cfg(feature="use_lifeguard")]
impl lifeguard::Recycleable for Growable {

    fn new() -> Self { Growable::new() }

    fn reset(&mut self) { }
}

impl Clone for Growable {

    fn clone(&self) -> Self {
        match self {
            &Growable::Some { len, ptr_alignment, .. } => Self::with_capacity(len, ptr_alignment),
            &Growable::None => Growable::None
        }
    }
}

impl fmt::Debug for Growable {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Growable::Some { len, ptr_alignment, ptr } =>
                write!(formatter, "Growable::Some {{ len: {:?}, ptr_alignment: {:?}, ptr: {:?} }}", len, ptr_alignment, ptr.as_ptr()),
            &Growable::None =>
                write!(formatter, "Growable::None")
        }
    }
}

impl Drop for Growable {

    fn drop(&mut self) {
        match self {
            &mut Growable::Some { len, ptr_alignment, ref mut ptr } => {
                unsafe {
                    Heap.dealloc(ptr.as_mut(), Layout::from_size_align_unchecked(len, ptr_alignment));
                }
            },
            &mut Growable::None => ()
        }
    }
}

impl Growable {

    /// Returns a new instance of Growable but does not allocate any memory on the heap yet.
    pub fn new() -> Self {
        Growable::None
    }

    /// Returns a new instance of Growable and allocates memory on the heap.
    /// In stable Rust it is possible to get a required
    /// pointer alignment for any type with [align_of](https://doc.rust-lang.org/std/mem/fn.align_of.html) function.
    pub fn with_capacity(len: usize, ptr_alignment: usize) -> Self {
        let mut temp = Self::new();
        temp.grow(len, ptr_alignment);
        temp
    }

    /// Returns the amount of memory allocated by this Growable.
    pub fn len(&self) -> usize {
        match self {
            &Growable::Some { len, .. } => len,
            &Growable::None => 0
        }
    }

    /// Returns allocated on the heap struct, an actual (re)allocation will be performed
    /// only if there is not enough space in this Growable
    /// or the pointer alignment is invalid.
    pub fn assign<T>(mut self, t: T) -> Reusable<T> where T: 'static {
        let result = self.assign_into(&t as &T, mem::align_of::<T>(), mem::size_of::<T>());
        mem::forget(t);
        mem::forget(self);
        result
    }

    /// Returns allocated on the heap struct, an actual (re)allocation will be performed
    /// only if there is not enough space in this Growable
    /// or the pointer alignment is invalid.
    /// Additionally stores meta pointer to the vtable creating trait object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use growable::*;
    ///   trait Answerer {
    ///
    ///       fn get_answer(&self) -> u32;
    ///   }
    ///
    ///   struct Foo;
    ///
    ///   impl Answerer for Foo {
    ///
    ///       fn get_answer(&self) -> u32 { 42 }
    ///   }
    ///
    ///   let growable = Growable::new();
    ///   let foo: Reusable<Answerer> = growable.assign_as_trait(Foo);
    ///   assert_eq!(foo.get_answer(), 42);
    /// ```
    pub fn assign_as_trait<T, U>(mut self, t: T) -> Reusable<U> where T: Unsize<U> + 'static, U: ?Sized + 'static {
        let result = self.assign_into(&t as &U, mem::align_of::<T>(), mem::size_of::<T>());
        mem::forget(t);
        mem::forget(self);
        result
    }

    fn grow(&mut self, len: usize, ptr_alignment: usize) {
        match self {
            &mut Growable::Some { len: ref mut curr_len, ptr_alignment: ref mut curr_ptr_alignment, ref mut ptr } if *curr_len < len || *curr_ptr_alignment < ptr_alignment => {
                let len = cmp::max(*curr_len, len);
                unsafe {
                    let layout_curr = Layout::from_size_align_unchecked(*curr_len, ptr_alignment);
                    let layout = Layout::from_size_align(len, ptr_alignment).expect("an invalid allocation request in self.grow");
                    *ptr = match Heap.realloc(ptr.as_mut(), layout_curr, layout) {
                         Ok(ptr) => Unique::new_unchecked(ptr),
                        Err(err) => {
                            if err.is_memory_exhausted() { Heap.oom(err) }
                            else { panic!("got an unexpected failure on a allocation attempt: {:?}", err); }
                        }
                    };
                    *curr_ptr_alignment = ptr_alignment;
                    *curr_len = len;
                }
            },
            &mut Growable::Some { .. } => (),
            &mut Growable::None => {
                unsafe {
                    let layout = Layout::from_size_align(len, ptr_alignment).expect("an invalid allocation request in self.grow");
                    let ptr = match Heap.alloc(layout) {
                         Ok(ptr) => Unique::new_unchecked(ptr),
                        Err(err) => {
                            if err.is_memory_exhausted() { Heap.oom(err) }
                                else { panic!("got an unexpected failure on a allocation attempt: {:?}", err); }
                        }
                    };
                    *self = Growable::Some {
                        len,
                        ptr_alignment,
                        ptr
                    };
                }
            }
        };
    }

    fn assign_into<T>(&mut self, t: &T, ptr_alignment: usize, len: usize) -> Reusable<T> where T: ?Sized + 'static {
        self.grow(next_highest_power_of_2(len), ptr_alignment);
        if let &mut Growable::Some { len, mut ptr, .. } = self {
            unsafe {
                let mut t = t as *const T as *mut T;
                let _thin = mem::transmute::<&mut *mut T, &mut *mut u8>(&mut t);
                ptr::copy(*_thin, ptr.as_mut(), len);
                *_thin = ptr.as_mut();
                Reusable {
                    len,
                    ptr_alignment,
                    ptr: Unique::new_unchecked(t)
                }
            }
        } else {
            unreachable!()
        }
    }
}

/// Growable with some data assigned to it. It behaves just
/// like default Box does (so it WILL free memory on drop) but also
/// could be freed manually, fetching Growable back.
pub struct Reusable<T: ?Sized> {
    len: usize,
    ptr_alignment: usize,
    ptr: Unique<T>
}

impl<T> ops::Deref for Reusable<T> where T: ?Sized {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<T> ops::DerefMut for Reusable<T> where T: ?Sized {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<T> Drop for Reusable<T> where T: ?Sized {

    fn drop(&mut self) {
        self.free_in_place();
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Reusable<U>> for Reusable<T> {}

impl<T> Reusable<T> where T: ?Sized {

    /// Performs drop call on the stored value and returns
    /// freed memory block back as a Growable struct.
    pub fn free(mut self) -> Growable {
        let growable = self.free_in_place();
        mem::forget(self);
        growable
    }

    fn free_in_place(&mut self) -> Growable {
        unsafe {
            let ptr = self.ptr.as_ptr();
            ptr::drop_in_place(ptr);
            Growable::Some {
                len: self.len,
                ptr_alignment: self.ptr_alignment,
                ptr: Unique::new_unchecked(ptr as *mut u8)
            }
        }
    }
}

#[inline]
fn next_highest_power_of_2(mut num: usize) -> usize {
    if 0 == num {
        return 0;
    }
    num -= 1;
    num |= num >> 0x01;
    num |= num >> 0x02;
    num |= num >> 0x04;
    num |= num >> 0x08;
    num |= num >> 0x10;
    if mem::size_of::<usize>() > 4 {
        num |= num >> 0x20;
    }
    num += 1;
    num
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_next_highest_power_of_2() {
        assert_eq!(next_highest_power_of_2(0), 0);
        assert_eq!(next_highest_power_of_2(1), 1);
        assert_eq!(next_highest_power_of_2(2), 2);
        assert_eq!(next_highest_power_of_2(3), 4);
        assert_eq!(next_highest_power_of_2(4), 4);
        assert_eq!(next_highest_power_of_2(5), 8);
        assert_eq!(next_highest_power_of_2(6), 8);
        assert_eq!(next_highest_power_of_2(7), 8);
        assert_eq!(next_highest_power_of_2(8), 8);
        assert_eq!(next_highest_power_of_2(15), 16);
        assert_eq!(next_highest_power_of_2(16), 16);
        assert_eq!(next_highest_power_of_2(17), 32);
        assert_eq!(next_highest_power_of_2(24), 32);
        assert_eq!(next_highest_power_of_2(45), 64);
        assert_eq!(next_highest_power_of_2(64), 64);
    }
}