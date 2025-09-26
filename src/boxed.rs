use core::{
    borrow,
    cmp::Ordering,
    fmt,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
};

use crate::{AllocRes, ArenaAllocatorImpl};

pub struct Box<'a, T: ?Sized>(&'a mut T);

impl<'a, T> Box<'a, T> {
    pub fn new_in<A: ArenaAllocatorImpl>(value: T, alloc: &'a A) -> AllocRes<Self> {
        alloc.alloc_val(value).map(|value_ref| Self(value_ref))
    }

    pub fn pin_in<A: ArenaAllocatorImpl>(value: T, alloc: &'a A) -> AllocRes<Pin<Self>> {
        Self::new_in(value, alloc).map(|boxed| boxed.into())
    }

    pub fn into_inner(b: Box<'_, T>) -> T {
        let raw = Self::into_raw(b);
        unsafe { ptr::read(raw) }
    }
}

impl<'a, T: ?Sized> Box<'a, T> {
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        Self(unsafe { &mut *ptr })
    }

    pub fn into_raw(b: Box<'_, T>) -> *mut T {
        let mut b = ManuallyDrop::new(b);
        b.deref_mut().0 as *mut T
    }

    pub fn leak(b: Box<'_, T>) -> &'a mut T {
        unsafe { &mut *Self::into_raw(b) }
    }
}

impl<'a, 'b, T: ?Sized + PartialEq> PartialEq<Box<'b, T>> for Box<'a, T> {
    #[inline]
    fn eq(&self, other: &Box<'b, T>) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<'a, 'b, T: ?Sized + PartialOrd> PartialOrd<Box<'b, T>> for Box<'a, T> {
    #[inline]
    fn partial_cmp(&self, other: &Box<'b, T>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    #[inline]
    fn lt(&self, other: &Box<'b, T>) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    #[inline]
    fn le(&self, other: &Box<'b, T>) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    #[inline]
    fn ge(&self, other: &Box<'b, T>) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    #[inline]
    fn gt(&self, other: &Box<'b, T>) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<'a, T: ?Sized + Ord> Ord for Box<'a, T> {
    #[inline]
    fn cmp(&self, other: &Box<'a, T>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<'a, T: ?Sized + Eq> Eq for Box<'a, T> {}

impl<'a, T: ?Sized> From<Box<'a, T>> for Pin<Box<'a, T>> {
    /// Converts a `Box<T>` into a `Pin<Box<T>>`.
    ///
    /// This conversion does not allocate on the heap and happens in place.
    fn from(boxed: Box<'a, T>) -> Self {
        // It's not possible to move or replace the insides of a `Pin<Box<T>>`
        // when `T: !Unpin`,  so it's safe to pin it directly without any
        // additional requirements.
        unsafe { Pin::new_unchecked(boxed) }
    }
}

impl<'a, T: fmt::Display + ?Sized> fmt::Display for Box<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'a, T: fmt::Debug + ?Sized> fmt::Debug for Box<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized> fmt::Pointer for Box<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // It's not possible to extract the inner Uniq directly from the Box,
        // instead we cast it to a *const which aliases the Unique
        let ptr: *const T = &**self;
        fmt::Pointer::fmt(&ptr, f)
    }
}

impl<'a, T: ?Sized> Deref for Box<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.0
    }
}

impl<'a, T: ?Sized> DerefMut for Box<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
    }
}

impl<'a, T: ?Sized> borrow::Borrow<T> for Box<'a, T> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<'a, T: ?Sized> borrow::BorrowMut<T> for Box<'a, T> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: ?Sized> AsRef<T> for Box<'a, T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<'a, T: ?Sized> AsMut<T> for Box<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<'a, T: ?Sized> Unpin for Box<'a, T> {}

/// This impl replaces unsize coercion.
impl<'a, T, const N: usize> From<Box<'a, [T; N]>> for Box<'a, [T]> {
    fn from(arr: Box<'a, [T; N]>) -> Box<'a, [T]> {
        let mut arr = ManuallyDrop::new(arr);
        let ptr = core::ptr::slice_from_raw_parts_mut(arr.as_mut_ptr(), N);
        unsafe { Box::from_raw(ptr) }
    }
}

/// This impl replaces unsize coercion.
impl<'a, T, const N: usize> TryFrom<Box<'a, [T]>> for Box<'a, [T; N]> {
    type Error = Box<'a, [T]>;
    fn try_from(slice: Box<'a, [T]>) -> Result<Box<'a, [T; N]>, Box<'a, [T]>> {
        if slice.len() == N {
            let mut slice = ManuallyDrop::new(slice);
            let ptr = slice.as_mut_ptr() as *mut [T; N];
            Ok(unsafe { Box::from_raw(ptr) })
        } else {
            Err(slice)
        }
    }
}
