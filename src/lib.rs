//! SecBox.
//!
//! This crate provides a security primitive, `SecBox`, which tries to limit the damage
//! of common vulnerabilities.
//!
//! # Vulerabilities
//!
//! - **Stack or local out-of-bound indexing:** You can usually use buffer overflow to read the
//! stack, but if you need to deref the element to get the data, you often cannot know how much to
//! offset by (that is, you don't know at what address the array starts). Scanning linearly is
//! unproductive (especially since the data doesn't line up) and quickly results in segfault.
//!
//! - **Partial memory dumps:** Partial memory dumps (e.g. page dumps or CPU cache dumps) are
//! avoided by discontinuity, which means that partial memory segments would rarely contain
//! interesting data.
//!
//! - **Swap RAM data leaks:** To avoid the memory being written to persistent memory (and thus
//! easier to access), we memlock the internal data, making sure that the data never leaves the
//! temporary memory.
//!
//! - **Read of uninitialized data:** Uninitialized reads is a rare bug in Rust, but it is common
//! in C and C++ and thus Rust bindings to libraries written in those. For this reason, we make
//! sure that the data overwritten it with zeros, and thus made unaccessible after free.
//!
//! - **Crash dump data leaks:** Due to zeroing data, crash dumps are often limited in exposure of
//! sensitive data.
//!
//! # NB!
//!
//! `SecBox` doesn't mean that the inner data is completely protected. You still need to make sure
//! it is handled properly and not leaked by other means.

#![feature(box_syntax, unique, core_intrinsics)]
#![warn(missing_docs)]

extern crate libc;

use std::ptr::{self, Unique};
use std::{mem, intrinsics, ops, fmt, slice};

/// A secure box.
///
/// This will make sure the internal memory is memlocked, and cleared when dropped.
///
/// While this is slower than e.g. having a secure string, it allows for better security due to
/// obfustication as well as no unsecure reallocation.
///
/// # Security measures
///
/// 1. Memlocking. This memlocks the inner data making sure the dataresident in memory.
/// 2. Volatile zeroing. This makes sure the data is overwritten when dropped, making it impossible
///    to read afterwards.
/// 3. Non linearity. If you have a vector of `SecBox`es, they will not necessarily be lined up,
///    which mean that  if an attacker can read some part of the memory, it will rarely make sense.
///
/// # An important note
///
/// Wrapping a primitive doesn't necessarily affect the inner data. Many primitves (like `Vec` and
/// `Box`) are simply wrappers around a pointer to the inner data. For this reason you need to wrap
/// the inner data (e.g. `Vec<SecBox<T>>` instaed of `SecBox<Vec<T>>`).
pub struct SecBox<T: ?Sized> {
    /// The inner pointer.
    ///
    /// We use a raw pointer so that we can handle the destructor manually.
    inner: Unique<T>,
}

impl<T: ?Sized> SecBox<T> {
    /// Create a new `SecBox`.
    ///
    /// If you want to construct a unsized SecBox, you should convert a `Box` through the `From`
    /// trait.
    #[inline(always)]
    pub fn new(inner: T) -> SecBox<T> where T: Sized {
        let res = SecBox {
            inner: unsafe { Unique::new(Box::into_raw(box mem::uninitialized::<T>())) },
        };

        // Lock the data.
        res.memlock();

        // We set the inner data after the memlock to make sure that the data doesn't leave the memory.
        unsafe {
            ptr::write(*res.inner, inner);
        }

        res
    }

    /// Get the inner value of this `SecBox`.
    ///
    /// Take care. This moves the value from a secure space to the stack, allowing the data to
    /// reside in swap RAM.
    pub fn into_inner(self) -> T where T: Sized {
        unsafe {
            // Read the inner.
            let res = ptr::read(*self.inner);
            // Zero it.
            ptr::write_volatile(*self.inner, mem::zeroed());
            // Unlock the memory.
            self.memunlock();

            res
        }
    }

    /// Memlock the inner data.
    fn memlock(&self) {
        unsafe {
            libc::mlock(&**self as *const T as *const libc::c_void,
                        mem::size_of_val(&**self) as libc::size_t);
        };
    }

    /// Memunlock the inner data.
    fn memunlock(&self) {
        unsafe {
            libc::munlock(&**self as *const T as *const libc::c_void,
                          mem::size_of_val(&**self) as libc::size_t);
        };
    }
}

impl<T: ?Sized + Clone> Clone for SecBox<T> {
    fn clone(&self) -> SecBox<T> {
        unsafe {
            let mut bx = SecBox::new(mem::uninitialized::<T>());

            // To avoid getting it outside the secure space, we clone inplace.
            bx.clone_from(self);

            bx
        }
    }

    fn clone_from(&mut self, src: &SecBox<T>) {
        (&mut **self).clone_from(src);
    }
}

impl<T: ?Sized> From<Box<T>> for SecBox<T> {
    fn from(from: Box<T>) -> SecBox<T> {
        let res = SecBox {
            inner: unsafe { Unique::new(Box::into_raw(from)) },
        };

        // Lock the data.
        res.memlock();

        res
    }
}

impl<T: ?Sized> ops::Deref for SecBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.inner.get() }
    }
}


impl<T: ?Sized> ops::DerefMut for SecBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.inner.get_mut() }
    }
}

impl<T: ?Sized> fmt::Display for SecBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "*******")
    }
}

impl<T: ?Sized> fmt::Debug for SecBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "*******")
    }
}

impl<T: ?Sized> Drop for SecBox<T> {
    fn drop(&mut self) {
        unsafe {
            // Drop the inner data.
            ptr::drop_in_place(*self.inner);
            // Zero the content.
            intrinsics::volatile_set_memory(*self.inner as *mut u8, 0, mem::size_of_val(&**self));

            // To avoid double-dropping, we convert our data into a byte string, which lacks of
            // destructors.
            let _buf = Box::from_raw(slice::from_raw_parts_mut(*self.inner as *mut u8, mem::size_of_val(&**self)));

            // Unlock the memory.
            self.memunlock();

            // _buf (the buffer) is freed.
        }
    }
}

#[cfg(test)]
mod test;
