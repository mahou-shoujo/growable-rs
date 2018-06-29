//! A growable, reusable box for Rust.
//!
//! This crate provides a custom Box type with matching API that also allows to reuse the same
//! heap to store different types with the minimal amount of allocations and is supposed to be
//! used with a custom, pool-based allocator of user's choice such as
//! [Lifeguard](https://crates.io/crates/lifeguard), but also can be used as a standalone solution.
//!
//! # Notes
//! 
//! This crate uses a lot of ground-breaking features of Rust and therefore
//! is only available on the latest Nightly build.

#![deny(missing_docs)]
#![feature(allocator_api, alloc, coerce_unsized, unsize)]

#[cfg(feature="use_lifeguard")] extern crate lifeguard;
extern crate alloc;

use std::cmp;
use std::fmt;
use std::marker::Unsize;
use std::mem;
use std::ops;
use std::ops::CoerceUnsized;
use std::ptr;
use std::ptr::NonNull;
use alloc::alloc::Global;
use alloc::allocator::{handle_alloc_error, Alloc, Excess, Layout};

/// A chunk of the heap memory that can be assigned with an arbitrary type.
/// Until assigned with some data it behaves similarly to a [`Box<\[u8; N\]>`],
/// it can be cloned and will be dropped if leaves the scope.
///
/// # Examples
///
/// First, let's spawn a new Growable. In this case no allocation will be performed.
///
/// ```
/// # use growable::*;
///   let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.consume(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
/// # let growable = arr.free();
/// # let arr: Reusable<[char; 6]> = growable.consume(['f', 'o', 'o', 'b', 'a', 'r']);
/// # assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// Now we can assign some data to it.
///
/// ```
/// # use growable::*;
/// # let growable = Growable::new();
///   let arr: Reusable<[char; 3]> = growable.consume(['f', 'o', 'o']);
///   assert_eq!(&*arr, &['f', 'o', 'o']);
/// # let growable = arr.free();
/// # let arr: Reusable<[char; 6]> = growable.consume(['f', 'o', 'o', 'b', 'a', 'r']);
/// # assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// No longer wanted data can be then freed on demand, fetching Growable back.
/// Then it could be assigned with some data again and so on.
///
/// ```
/// # use growable::*;
/// # let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.consume(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
///   let growable = arr.free();
///   let arr: Reusable<[char; 6]> = growable.consume(['f', 'o', 'o', 'b', 'a', 'r']);
///   assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
/// 
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
pub struct Growable {
    len: usize,
    ptr_alignment: usize,
    ptr: NonNull<u8>,
}

impl Clone for Growable {

    #[inline]
    fn clone(&self) -> Self {
        Self::with_capacity(self.len, self.ptr_alignment)
    }
}

impl fmt::Pointer for Growable {
    
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result { fmt::Pointer::fmt(&self.ptr, formatter) }
}

impl fmt::Debug for Growable {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.len {
            0 => write!(formatter, "Growable::None"),
            _ => write!(formatter, "Growable::Some<len = {:?}, align = {:?}>({:p})", self.len, self.ptr_alignment, self.ptr),
        }
    }
}

impl Default for Growable {

    #[inline]
    fn default() -> Self {
        Growable::new()
    }
}

impl Drop for Growable {

    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                Global.dealloc(self.ptr, Layout::from_size_align_unchecked(self.len, self.ptr_alignment));
            }
        }
    }
}

impl Growable {
    
    /// Returns a new instance of `Growable` but does not allocate any memory on the heap yet.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use growable::*;
    ///   let _ = Growable::new();
    /// ```
    /// 
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn new() -> Self {
        Growable::with_capacity(0, 1)
    }

    /// Returns a new instance of `Growable` with memory already allocated on the heap suitable to
    /// store an instance of a given type T.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use growable::*;
    ///   struct Foo {
    ///       a: u8,
    ///       b: usize,
    ///       c: (),
    ///   }
    ///   let _ = Growable::with_capacity_for_type::<Foo>();
    /// ```
    /// 
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn with_capacity_for_type<T>() -> Self { Self::with_capacity(mem::size_of::<T>(), mem::align_of::<T>()) }

    /// Returns a new instance of `Growable` with memory already allocated on the heap.
    ///
    /// # Panics
    /// 
    /// * `ptr_alignment` is not a power of two.
    /// * `len` overflows after being rounded up to the nearest multiple of the alignment.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use growable::*;
    ///   let _ = Growable::with_capacity(256, 16);
    /// ```
    /// 
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn with_capacity(len: usize, ptr_alignment: usize) -> Self {
        unsafe {
            let Excess(ptr, len) = if len != 0 {
                let layout = Layout::from_size_align(len, ptr_alignment).expect("Growable::with_capacity: invalid layout");
                Global.alloc_excess(layout).unwrap_or_else(|_| handle_alloc_error(layout))
            } else {
                assert!(ptr_alignment.is_power_of_two(), "Growable::with_capacity: alignment must be a power of two");
                Excess(NonNull::<u8>::dangling(), 0)
            };
            Growable {
                len,
                ptr_alignment,
                ptr,
            }
        }
    }

    /// Returns true if no memory has been allocated yet.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns the amount of memory allocated by this `Growable`.
    /// 
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn len(&self) -> usize { self.len }

    /// Returns the alignment.
    #[inline]
    pub fn alignment(&self) -> usize { self.ptr_alignment }

    /// Places an instance of `T` on the heap, an actual (re)allocation will be performed
    /// only if there is not enough space or the pointer alignment is invalid.
    /// 
    /// # Notes
    /// 
    /// Might trigger `oom()` handler.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use growable::*;
    ///   let growable = Growable::with_capacity(128, 8);
    ///   let num = growable.consume(0usize);
    ///   assert_eq!(*num, 0usize);
    /// ```
    #[inline]
    pub fn consume<T>(mut self, t: T) -> Reusable<T> {
        self.grow(mem::size_of::<T>(), mem::align_of::<T>());
        self.copy(t)
    }

    fn grow(&mut self, len: usize, ptr_alignment: usize) {
        // NB: len is valid or zero, ptr_alignment is always valid.
        if self.len == 0 {
            // Growing from zero length could be done with Growable::with_capacity call.
            *self = Growable::with_capacity(len, ptr_alignment);
            return;
        }
        if self.len >= len &&
           self.ptr_alignment >= ptr_alignment {
            // No allocation is required.
            return;
        }
        
        let len = cmp::max(self.len, len);
        // NB: Could be a bug if there is a way to define a ZST with align_of() greater than one?!
        debug_assert_ne!(len, 0, "Growable::grow: realloc to zero");
        unsafe {
            let layout_curr = Layout::from_size_align_unchecked(self.len, self.ptr_alignment);
            let layout = Layout::from_size_align_unchecked(len, ptr_alignment);
            // If the alignment is the same we can try to grow in place.
            let growed_in_place =
                layout.align() == layout_curr.align() &&
                    Global.grow_in_place(self.ptr, layout_curr, len).is_ok();
            if !growed_in_place {
                // Oops, a reallocation is required.
                let Excess(ptr, len) = Global.realloc_excess(self.ptr, layout_curr, len).unwrap_or_else(|_| handle_alloc_error(layout));
                self.len = len;
                self.ptr_alignment = ptr_alignment;
                self.ptr = ptr;
            } else {
                // On successful grow_in_place we only need to update len.
                self.len = len;
            }
        }
    }
    
    fn copy<T>(self, t: T) -> Reusable<T> {
        // NB: len is at least equal to size_of::<T>(), ptr_alignment is at least equal to align_of::<T>().  
        let result = unsafe {
            let ptr_raw = self.ptr.cast::<T>().as_ptr();
            ptr_raw.write(t);
            let ptr = NonNull::new_unchecked(ptr_raw);
            Reusable {
                len: self.len,
                ptr_alignment: self.ptr_alignment,
                ptr,
            }
        };
        mem::forget(self);
        result
    }
}

/// A reusable box. It behaves just
/// like the default [`Box`] (hence it WILL free memory on drop) but also
/// could be freed manually, fetching a [`Growable`] back.
/// 
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
/// [`Growable`]: struct.Growable.html
pub struct Reusable<T: ?Sized> {
    len: usize,
    ptr_alignment: usize,
    ptr: NonNull<T>,
}

impl<T> Clone for Reusable<T>
    where
        T: ?Sized + Clone, {
    
    fn clone(&self) -> Self {
        let growable = Growable::with_capacity_for_type::<T>();
        growable.consume(T::clone(&*self))
    }
}

impl<T: ?Sized> ops::Deref for Reusable<T> {

    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            self.ptr.as_ref()
        }
    }
}

impl<T: ?Sized> ops::DerefMut for Reusable<T> {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.ptr.as_mut()
        }
    }
}

impl<T> fmt::Pointer for Reusable<T>
    where
        T: ?Sized, {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result { fmt::Pointer::fmt(&self.ptr, formatter) }
}

impl<T> fmt::Debug for Reusable<T>
    where
        T: ?Sized + fmt::Debug, {

    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let t: &T = &*self;
        fmt::Debug::fmt(t, formatter)
    }
}

impl<T: ?Sized> Drop for Reusable<T> {

    fn drop(&mut self) {
        self.free_in_place();
    }
}

impl<T, U> CoerceUnsized<Reusable<U>> for Reusable<T>
    where
        T: ?Sized + Unsize<U>,
        U: ?Sized, {
}

impl<T: ?Sized> Reusable<T> {

    /// Drops the value and returns the memory back as a [`Growable`].
    /// 
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn free(mut self) -> Growable {
        let growable = self.free_in_place();
        mem::forget(self);
        growable
    }

    #[inline]
    fn free_in_place(&mut self) -> Growable {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            Growable {
                len: self.len,
                ptr_alignment: self.ptr_alignment,
                ptr: self.ptr.cast(),
            }
        }
    }
}
