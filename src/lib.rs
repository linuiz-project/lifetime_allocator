#![no_std]

use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[derive(Debug, thiserror::Error)]
#[error("an allocation error occurred")]
pub struct AllocError;

pub struct AllocPtr<'a, T: ?Sized, A: Allocator> {
    obj: NonNull<T>,
    allocator: A,
    marker: PhantomData<&'a ()>,
}

impl<T: ?Sized, A: Allocator> AllocPtr<'_, T, A> {
    /// # Safety
    ///
    /// - `obj` must be a dereferencable as `T`.
    /// - `allocator` must be the [`Allocator`] that `obj` was allocated from.
    pub unsafe fn new(obj: NonNull<T>, allocator: A) -> Self {
        Self {
            obj,
            allocator,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, A: Allocator> Deref for AllocPtr<'_, T, A> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.obj.as_ref() }
    }
}

impl<T: ?Sized, A: Allocator> DerefMut for AllocPtr<'_, T, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.obj.as_mut() }
    }
}

impl<'a, T: ?Sized, A: Allocator> Drop for AllocPtr<'a, T, A> {
    fn drop(&mut self) {
        unsafe {
            self.allocator.deallocate(self);
        }
    }
}

pub unsafe trait Allocator: Sized {
    fn allocate<'a, T: Default>(&'a self) -> Result<AllocPtr<'a, T, Self>, AllocError>;

    fn allocate_uninit<'a, T>(&'a self) -> Result<AllocPtr<'a, MaybeUninit<T>, Self>, AllocError>;

    #[cfg(feature = "zerocopy")]
    fn allocate_zeroed<'a, T: zerocopy::FromZeros>(
        &'a self,
    ) -> Result<AllocPtr<'a, T, Self>, AllocError>;

    fn allocate_slice<'a, T>(&'a self) -> Result<AllocPtr<'a, [MaybeUninit<T>], Self>, AllocError>;

    fn allocate_uninit_slice<'a, T>(
        &'a self,
    ) -> Result<AllocPtr<'a, [MaybeUninit<T>], Self>, AllocError>;

    unsafe fn deallocate<'a, 'b: 'a, T: ?Sized>(&'a self, alloc_ptr: &AllocPtr<'b, T, Self>);
}
