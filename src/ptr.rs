
use alloc::boxed::Box;
use core::marker::PhantomData;
use core::ptr::NonNull;

#[repr(transparent)]
pub struct Own<T>
where
    T: ?Sized,
{
    pub ptr: NonNull<T>,
}

unsafe impl<T> Send for Own<T> where T: ?Sized {}

unsafe impl<T> Sync for Own<T> where T: ?Sized {}

impl<T> Copy for Own<T> where T: ?Sized {}

impl<T> Clone for Own<T>
where
    T: ?Sized,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Own<T>
where
    T: ?Sized,
{
    pub fn new(ptr: Box<T>) -> Self {
        Own {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(ptr)) },
        }
    }

    pub fn cast<U: CastTo>(self) -> Own<U::Target> {
        Own {
            ptr: self.ptr.cast(),
        }
    }

    pub unsafe fn boxed(self) -> Box<T> {
        Box::from_raw(self.ptr.as_ptr())
    }

    pub fn by_ref(&self) -> Ref<T> {
        Ref {
            ptr: self.ptr,
            lifetime: PhantomData,
        }
    }

    pub fn by_mut(&mut self) -> Mut<T> {
        Mut {
            ptr: self.ptr,
            lifetime: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct Ref<'a, T>
where