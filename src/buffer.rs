use core::{array, cell::UnsafeCell, ptr};

pub(crate) use heap_::*;

pub(crate) trait Buffer<T> {
    fn as_ptr(&self) -> *const T;
    fn as_mut_ptr(&self) -> *mut T;
    fn as_slice(&self) -> &[T] {
        unsafe { &*ptr::slice_from_raw_parts(self.as_ptr(), self.len()) }
    }
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

pub(crate) struct StackBuf<const N: usize, T> {
    inner: UnsafeCell<[T; N]>,
}

impl<const N: usize, T: Default> StackBuf<N, T> {
    pub(crate) fn new() -> Self {
        Self {
            inner: array::from_fn(|_| T::default()).into(),
        }
    }
}

impl<const N: usize, T> Buffer<T> for StackBuf<N, T> {
    fn as_ptr(&self) -> *const T {
        self.inner.get() as *const T
    }

    fn as_mut_ptr(&self) -> *mut T {
        self.inner.get() as *mut T
    }

    fn len(&self) -> usize {
        N
    }
}

impl<const N: usize, T: Default> Default for StackBuf<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "alloc")]
mod heap_ {
    use super::*;
    use alloc::boxed::Box;

    pub(crate) struct HeapBuf<T> {
        inner: UnsafeCell<Box<[T]>>,
    }

    impl<T> Buffer<T> for HeapBuf<T> {
        fn as_ptr(&self) -> *const T {
            self.inner.get() as *const T
        }

        fn as_mut_ptr(&self) -> *mut T {
            self.inner.get() as *mut T
        }
    }

    impl<T: Default> HeapBuf<T> {
        pub(crate) fn new(size: usize) -> Self {
            Self {
                inner: (0..size).map(|_| T::default()).collect::<Box<[T]>>().into(),
            }
        }
    }
}
