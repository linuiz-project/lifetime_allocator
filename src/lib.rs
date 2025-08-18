#![no_std]

use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
use zerocopy::FromZeros;

pub struct AllocPtr<'a, T: ?Sized, A: Allocator> {
    obj: NonNull<T>,
    allocator: A,
    marker: PhantomData<&'a ()>,
}

impl<T: ?Sized, A: Allocator> AllocPtr<'_, T, A> {
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
    type Error;

    fn allocate<'a, T: Default>(&'a self) -> Result<AllocPtr<'a, T, Self>, Self::Error>;

    fn allocate_uninit<'a, T>(&'a self) -> Result<AllocPtr<'a, MaybeUninit<T>, Self>, Self::Error>;

    fn allocate_zeroed<'a, T: FromZeros>(&'a self) -> Result<AllocPtr<'a, T, Self>, Self::Error>;

    fn allocate_slice<'a, T>(&'a self)
    -> Result<AllocPtr<'a, [MaybeUninit<T>], Self>, Self::Error>;

    fn allocate_uninit_slice<'a, T>(
        &'a self,
    ) -> Result<AllocPtr<'a, [MaybeUninit<T>], Self>, Self::Error>;

    unsafe fn deallocate<'a, 'b: 'a, T: ?Sized>(&'a self, alloc_ptr: &AllocPtr<'b, T, Self>);
}
