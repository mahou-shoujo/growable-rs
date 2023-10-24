//! A growable, reusable box for Rust.
//!
//! This crate provides a custom Box type with matching API that also allows to reuse the same
//! memory block to store different types with the minimal amount of allocations and is supposed to be
//! used with a pool-based allocator such as [`GrowablePool`].
//!
//! # Notes
//!
//! This crate uses a lot of ground-breaking features of Rust and therefore
//! is only available on the latest Nightly build.
//!
//! [`GrowablePool`]: struct.GrowablePool.html

#![deny(missing_docs, missing_debug_implementations)]
#![feature(allocator_api, coerce_unsized, slice_ptr_get, unsize)]

use std::{
    alloc::{handle_alloc_error, Allocator, Global, Layout},
    cmp,
    collections::VecDeque,
    fmt,
    marker::Unsize,
    mem,
    ops::{self, CoerceUnsized},
    ptr::{self, NonNull},
};

/// A customizable [`GrowablePool`] builder.
///
/// # Examples
///
/// ```
/// # use growable::*;
///   let _ = GrowablePool::builder()
///       .with_default_capacity(128)
///       .with_default_ptr_alignment(16)
///       .with_capacity(512)
///       .enable_overgrow(true)
///       .build();
/// ```
///
/// [`GrowablePool`]: struct.GrowablePool.html
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrowablePoolBuilder {
    len: usize,
    per_growable_len: usize,
    per_growable_ptr_alignment: usize,
    overgrow: bool,
}

impl Default for GrowablePoolBuilder {
    fn default() -> Self {
        GrowablePoolBuilder::new()
    }
}

impl GrowablePoolBuilder {
    /// Creates a new pool builder with default options.
    pub fn new() -> Self {
        GrowablePoolBuilder {
            len: 0,
            per_growable_len: 8,
            per_growable_ptr_alignment: 8,
            overgrow: true,
        }
    }

    /// If set to `false` all returning [`Growable`] will be dropped if
    /// there is not enough free space available in a pool.
    ///
    /// [`Growable`]: struct.Growable.html
    pub fn enable_overgrow(&mut self, enable: bool) -> &mut Self {
        self.overgrow = enable;
        self
    }

    /// Sets the default capacity for each allocated [`Growable`].
    ///
    /// [`Growable`]: struct.Growable.html
    pub fn with_default_capacity(&mut self, len: usize) -> &mut Self {
        self.per_growable_len = len;
        self
    }

    /// Sets the default ptr alignment for each allocated [`Growable`].
    ///
    /// [`Growable`]: struct.Growable.html
    pub fn with_default_ptr_alignment(&mut self, ptr_alignment: usize) -> &mut Self {
        self.per_growable_ptr_alignment = ptr_alignment;
        self
    }

    /// Sets a pool capacity used for every pool reallocation. Note that with `overgrow`
    /// enabled it is possible for the pool to grow beyond this capacity.
    /// If set to zero the pool will only allocate a [`Growable`] on an explicit allocation request.
    ///
    /// [`Growable`]: struct.Growable.html
    pub fn with_capacity(&mut self, capacity: usize) -> &mut Self {
        self.len = capacity;
        self
    }

    /// Creates a new [`GrowablePool`] using this builder.
    ///
    /// [`GrowablePool`]: struct.GrowablePool.html
    pub fn build(&self) -> GrowablePool {
        let vec = {
            let default = Growable::with_capacity(self.per_growable_len, self.per_growable_ptr_alignment);
            let mut vec = VecDeque::with_capacity(self.len);
            vec.resize(self.len, default);
            vec
        };
        GrowablePool {
            len: self.len,
            per_growable_len: self.per_growable_len,
            per_growable_ptr_alignment: self.per_growable_ptr_alignment,
            overgrow: self.overgrow,
            vec,
        }
    }
}

/// A pool of [`Growable`] objects. Unlike a typical Arena-based allocator it probably
/// will not be able to decrease a memory fragmentation or provide some strong
/// guarantees about frequency of allocations in your code but instead
/// can be used to reduce the total amount of allocations in an amortized way
/// by reusing the same memory to store different objects.
///
/// # Examples
///
/// Let's start off by creating a default [`GrowablePool`].
///
/// ```
/// # use growable::*;
///   // A default pool will not allocate anything just yet though.
///   let mut pool = GrowablePool::default();
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3, 4, 5, 6]);
/// # assert_eq!(&*arr, &[1, 2, 3, 4, 5, 6]);
/// # pool.free(arr);
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3]);
/// # assert_eq!(&*arr, &[1, 2, 3]);
/// ```
///
/// We can now use it to allocate some data and do something with it.
///
/// ```
/// # use growable::*;
/// # let mut pool = GrowablePool::default();
///   // Actually allocates a block capable to store at least this 6 bytes.
///   let arr: Reusable<[u8]> = pool.allocate([1, 2, 3, 4, 5, 6]);
///   assert_eq!(&*arr, &[1, 2, 3, 4, 5, 6]);
/// # pool.free(arr);
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3]);
/// # assert_eq!(&*arr, &[1, 2, 3]);
/// ```
///
/// An then return it back to the pool..
///
/// ```
/// # use growable::*;
/// # let mut pool = GrowablePool::default();
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3, 4, 5, 6]);
/// # assert_eq!(&*arr, &[1, 2, 3, 4, 5, 6]);
///   pool.free(arr);
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3]);
/// # assert_eq!(&*arr, &[1, 2, 3]);
/// ```
///
/// .. and reuse the same heap to store something else.
///
/// ```
/// # use growable::*;
/// # let mut pool = GrowablePool::default();
/// # let arr: Reusable<[u8]> = pool.allocate([1, 2, 3, 4, 5, 6]);
/// # assert_eq!(&*arr, &[1, 2, 3, 4, 5, 6]);
/// # pool.free(arr);
///   // No allocation is required.
///   let arr: Reusable<[u8]> = pool.allocate([1, 2, 3]);
///   assert_eq!(&*arr, &[1, 2, 3]);
/// ```
///
/// [`Growable`]: struct.Growable.html
/// [`GrowablePool`]: struct.GrowablePool.html
pub struct GrowablePool {
    len: usize,
    per_growable_len: usize,
    per_growable_ptr_alignment: usize,
    overgrow: bool,
    vec: VecDeque<Growable>,
}

impl Clone for GrowablePool {
    fn clone(&self) -> Self {
        GrowablePoolBuilder::default()
            .with_default_capacity(self.per_growable_len)
            .with_default_ptr_alignment(self.per_growable_ptr_alignment)
            .with_capacity(self.len)
            .enable_overgrow(self.overgrow)
            .build()
    }
}

impl fmt::Debug for GrowablePool {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "GrowablePool {{ .. {} more allocations available .. }}", self.vec.len())
    }
}

impl Default for GrowablePool {
    fn default() -> Self {
        GrowablePool::new()
    }
}

impl GrowablePool {
    /// Creates a new pool with default options.
    ///
    /// # Notes
    ///
    /// See [`GrowablePoolBuilder`] for advanced configuration.
    ///
    /// [`GrowablePoolBuilder`]: struct.GrowablePoolBuilder.html
    pub fn new() -> Self {
        GrowablePoolBuilder::default().build()
    }

    /// Creates a new pool builder with default options.
    pub fn builder() -> GrowablePoolBuilder {
        GrowablePoolBuilder::default()
    }

    /// Returns true if a reallocation will be needed to allocate an another one object.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the current amount of allocations that this pool can provide without a reallocation.
    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Allocates a new [`Reusable`] from the pool.
    ///
    /// # Notes
    ///
    /// If no [`Growable`] is available for allocation, the entire pool will be reallocated.
    ///
    /// [`Growable`]: struct.Growable.html
    /// [`Reusable`]: struct.Reusable.html
    #[inline]
    pub fn allocate<T>(&mut self, t: T) -> Reusable<T> {
        match self.vec.pop_front() {
            Some(growable) => growable.consume(t),
            None => {
                let default = Growable::with_capacity(self.per_growable_len, self.per_growable_ptr_alignment);
                self.vec.resize(cmp::max(self.len, 1), default);
                self.allocate(t)
            },
        }
    }

    /// Returns the [`Reusable`] back to the pool, marking it
    /// available for a next allocation.
    ///
    /// # Notes
    ///
    /// With overgrow disabled the [`Growable`] might be dropped entirely if
    /// there is not enough free space available in the pool.
    ///
    /// [`Growable`]: struct.Growable.html
    /// [`Reusable`]: struct.Reusable.html
    #[inline]
    pub fn free<T>(&mut self, t: Reusable<T>)
    where
        T: ?Sized,
    {
        if !self.overgrow && self.vec.len() >= self.len {
            return;
        }
        self.vec.push_front(Reusable::free(t));
    }
}

/// A chunk of the heap memory that can be assigned with an arbitrary type.
///
/// # Examples
///
/// First, let's spawn a new [`Growable`]. In this case no allocation will be performed.
///
/// ```
/// # use growable::*;
///   let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.consume(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
/// # let growable = Reusable::free(arr);
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
/// # let growable = Reusable::free(arr);
/// # let arr: Reusable<[char; 6]> = growable.consume(['f', 'o', 'o', 'b', 'a', 'r']);
/// # assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// No longer wanted data can be then freed on demand, fetching Growable back.
/// Then it might be assigned with some data again and so on.
///
/// ```
/// # use growable::*;
/// # let growable = Growable::new();
/// # let arr: Reusable<[char; 3]> = growable.consume(['f', 'o', 'o']);
/// # assert_eq!(&*arr, &['f', 'o', 'o']);
///   let growable = Reusable::free(arr);
///   let arr: Reusable<[char; 6]> = growable.consume(['f', 'o', 'o', 'b', 'a', 'r']);
///   assert_eq!(&*arr, &['f', 'o', 'o', 'b', 'a', 'r']);
/// ```
///
/// [`Growable`]: struct.Growable.html
pub struct Growable {
    len: usize,
    ptr_alignment: usize,
    ptr: NonNull<u8>,
}

unsafe impl Send for Growable {}

unsafe impl Sync for Growable {}

impl Clone for Growable {
    #[inline]
    fn clone(&self) -> Self {
        Self::with_capacity(self.len, self.ptr_alignment)
    }
}

impl fmt::Pointer for Growable {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, formatter)
    }
}

impl fmt::Debug for Growable {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.len {
            0 => write!(formatter, "Growable::None"),
            _ => {
                write!(
                    formatter,
                    "Growable::Some<len = {:?}, align = {:?}>({:p})",
                    self.len, self.ptr_alignment, self.ptr
                )
            },
        }
    }
}

impl Default for Growable {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Growable {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                Global.deallocate(self.ptr, Layout::from_size_align_unchecked(self.len, self.ptr_alignment));
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
        Self::with_capacity(0, 1)
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
    pub fn with_capacity_for_type<T>() -> Self {
        Self::with_capacity(mem::size_of::<T>(), mem::align_of::<T>())
    }

    /// Returns a new instance of `Growable` with memory already allocated on the heap.
    ///
    /// # Panics
    ///
    /// * `ptr_alignment` is not a power of two.
    /// * `len` overflows after being rounded up to the nearest multiple of the alignment.
    ///
    /// # Notes
    ///
    /// Might trigger `alloc_error` handler.
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
        let ptr = if len != 0 {
            let layout = Layout::from_size_align(len, ptr_alignment).expect("Growable::with_capacity: invalid layout");
            Global.allocate(layout).map_or_else(|_| handle_alloc_error(layout), |ptr| ptr.as_non_null_ptr())
        } else {
            assert!(ptr_alignment.is_power_of_two(), "Growable::with_capacity: alignment must be a power of two");
            NonNull::<u8>::dangling()
        };
        Growable {
            len,
            ptr_alignment,
            ptr,
        }
    }

    /// Returns true if no memory has been allocated yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the amount of memory allocated by this `Growable`.
    ///
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the alignment.
    #[inline]
    pub fn alignment(&self) -> usize {
        self.ptr_alignment
    }

    /// Places an instance of `T` on the heap, an actual (re)allocation will be performed
    /// only if there is not enough space or the pointer alignment is invalid.
    ///
    /// # Notes
    ///
    /// Might trigger `alloc_error` handler.
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
            // Growing from zero length can be done with Growable::with_capacity call.
            *self = Self::with_capacity(len, ptr_alignment);
            return;
        }
        if self.len >= len && self.ptr_alignment >= ptr_alignment {
            // No allocation is required.
            return;
        }

        let len = cmp::max(self.len, len);
        // NB: Could be a bug if there is a way to define a ZST with align_of() greater than one?!
        assert_ne!(len, 0, "Growable::grow: realloc to zero");
        unsafe {
            let layout_curr = Layout::from_size_align_unchecked(self.len, self.ptr_alignment);
            let layout = Layout::from_size_align_unchecked(len, ptr_alignment);
            // If the alignment is the same we can try to grow in place.
            let ptr = if layout.align() == layout_curr.align() {
                Global.grow(self.ptr, layout_curr, layout)
            } else {
                // Oops, a reallocation is required.
                Global.deallocate(self.ptr, layout_curr);
                Global.allocate(layout)
            }
            .map_or_else(|_| handle_alloc_error(layout), |ptr| ptr.as_non_null_ptr());
            self.len = len;
            self.ptr_alignment = ptr_alignment;
            self.ptr = ptr;
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

/// A reusable box. It behaves just like the default [`Box`] (and it WILL free memory on drop)
/// but it is also possible to free it manually, fetching a [`Growable`] back.
///
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
/// [`Growable`]: struct.Growable.html
pub struct Reusable<T: ?Sized> {
    len: usize,
    ptr_alignment: usize,
    ptr: NonNull<T>,
}

unsafe impl<T> Send for Reusable<T> where T: Send + ?Sized {}

unsafe impl<T> Sync for Reusable<T> where T: Sync + ?Sized {}

impl<T> Clone for Reusable<T>
where
    T: ?Sized + Clone,
{
    fn clone(&self) -> Self {
        let growable = Growable::with_capacity_for_type::<T>();
        growable.consume(T::clone(self))
    }
}

impl<T: ?Sized> ops::Deref for Reusable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> ops::DerefMut for Reusable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> fmt::Pointer for Reusable<T>
where
    T: ?Sized,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        fmt::Pointer::fmt(&self.ptr, formatter)
    }
}

impl<T> fmt::Debug for Reusable<T>
where
    T: ?Sized + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let t: &T = self;
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
    U: ?Sized,
{
}

impl<T: ?Sized> Reusable<T> {
    /// Drops the value and returns the memory back as a [`Growable`].
    ///
    /// [`Growable`]: struct.Growable.html
    #[inline]
    pub fn free(mut this: Self) -> Growable {
        let growable = this.free_in_place();
        mem::forget(this);
        growable
    }

    /// Moves the value out of this [`Reusable`] without dropping it and then
    /// returns it back with [`Growable`].
    ///
    /// [`Growable`]: struct.Growable.html
    /// [`Reusable`]: struct.Reusable.html
    #[inline]
    pub fn free_move(this: Self) -> (T, Growable)
    where
        T: Sized,
    {
        unsafe {
            let t = ptr::read(this.ptr.as_ptr());
            let growable = Growable {
                len: this.len,
                ptr_alignment: this.ptr_alignment,
                ptr: this.ptr.cast(),
            };
            mem::forget(this);
            (t, growable)
        }
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

/// Replaces the value, dropping the old one but not the memory associated with it.
///
/// # Notes
///
/// Has the same result as a manual call to [`Reusable::free`] and then [`Growable::consume`].
///
/// [`Reusable::free`]: struct.Reusable.html#method.free
/// [`Growable::consume`]: struct.Growable.html#method.consume
#[inline]
pub fn replace<T, U>(this: Reusable<T>, u: U) -> Reusable<U>
where
    T: ?Sized,
{
    Reusable::free(this).consume(u)
}
