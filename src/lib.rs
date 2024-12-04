#![no_std]

use core::{
    fmt::{Debug, Pointer},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

/// An atomic wrapper around [`core::ptr::NonNull`].
///
/// AtomicNoneNull is marked as `repr(transparent)` for [`core::sync::atomic::AtomicPtr`].
#[repr(transparent)]
pub struct AtomicNonNull<T> {
    ptr: AtomicPtr<T>,
}

impl<T> Debug for AtomicNonNull<T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.ptr, f)
    }
}

impl<T> Pointer for AtomicNonNull<T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Pointer::fmt(&self.ptr, f)
    }
}

impl<T> AtomicNonNull<T> {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline]
    pub const fn new(ptr: *mut T) -> Option<Self> {
        if !ptr.is_null() {
            // SAFETY: `ptr` is not null.
            unsafe { Some(Self::new_unchecked(ptr)) }
        } else {
            None
        }
    }

    /// # Safety
    /// `ptr` cannot be null.
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: AtomicPtr::new(ptr),
        }
    }

    #[inline]
    pub const fn from_non_null(ptr: NonNull<T>) -> Self {
        // SAFETY: `NonNull` is never null.
        unsafe { Self::new_unchecked(ptr.as_ptr()) }
    }

    /// Look at [`core::ptr::with_exposed_provenance`] for more information.
    #[inline]
    pub fn with_exposed_provenance(addr: usize) -> Option<Self> {
        Self::new(ptr::with_exposed_provenance_mut(addr))
    }

    /// Look at [`core::ptr::without_provenance`] for more information.
    #[inline]
    pub fn without_provenance(addr: usize) -> Option<Self> {
        Self::new(ptr::without_provenance_mut(addr))
    }

    /// Look at [`core::ptr::dangling`] for more information.
    #[inline]
    pub fn dangling() -> Self {
        // SAFETY: a dangling pointer is never null.
        unsafe { Self::new_unchecked(ptr::dangling_mut()) }
    }

    /// Look at [`core::sync::atomic::AtomicPtr::load`] for more information.
    #[inline]
    pub fn load(&self, order: Ordering) -> NonNull<T> {
        // SAFETY: `self` is always non-null.
        unsafe { NonNull::new_unchecked(self.ptr.load(order)) }
    }

    /// Look at [`core::sync::atomic::AtomicPtr::store`] for more information.
    #[inline]
    pub fn store(&self, value: NonNull<T>, order: Ordering) {
        self.ptr.store(value.as_ptr(), order);
    }

    /// Look at [`core::sync::atomic::AtomicPtr::swap`] for more information.
    #[inline]
    pub fn swap(&self, other: NonNull<T>, order: Ordering) -> Self {
        // SAFETY: the old value of `self` will always be non-null.
        unsafe { Self::new_unchecked(self.ptr.swap(other.as_ptr(), order)) }
    }

    /// Look at [`core::sync::atomic::AtomicPtr::compare_exchange`] for more information.
    #[inline]
    pub fn compare_exchange(
        &self,
        current: NonNull<T>,
        new: NonNull<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        // SAFETY: `current` and `new` and `self` cannot be null.
        unsafe {
            self.ptr
                .compare_exchange(current.as_ptr(), new.as_ptr(), success, failure)
                .map(|ptr| Self::new_unchecked(ptr))
                .map_err(|ptr| Self::new_unchecked(ptr))
        }
    }

    /// Look at [`core::sync::atomic::AtomicPtr::compare_exchange_weak`] for more information.
    #[inline]
    pub fn compare_exchange_weak(
        &self,
        current: NonNull<T>,
        new: NonNull<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<Self, Self> {
        // SAFETY: `current` and `new` and `self` cannot be null.
        unsafe {
            self.ptr
                .compare_exchange_weak(current.as_ptr(), new.as_ptr(), success, failure)
                .map(|ptr| Self::new_unchecked(ptr))
                .map_err(|ptr| Self::new_unchecked(ptr))
        }
    }

    /// Look at [`core::sync::atomic::AtomicPtr::fetch_update`] for more information.
    #[inline]
    pub fn fetch_update(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut f: impl FnMut(NonNull<T>) -> Option<NonNull<T>>,
    ) -> Result<Self, Self> {
        // SAFETY: `self` and the return value of `f` must be non-null.
        unsafe {
            self.ptr
                .fetch_update(set_order, fetch_order, |ptr| {
                    f(NonNull::new_unchecked(ptr)).map(|ptr| ptr.as_ptr())
                })
                .map(|ptr| Self::new_unchecked(ptr))
                .map_err(|ptr| Self::new_unchecked(ptr))
        }
    }
}
