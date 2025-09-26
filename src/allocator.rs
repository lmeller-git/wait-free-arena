use core::{
    alloc::Layout,
    ptr::{self, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{AllocError, AllocErrorKind, AllocRes, buffer::Buffer};
#[cfg(feature = "alloc")]
pub use heap_::*;
pub use stack_::*;

pub trait ArenaAllocatorImpl {
    fn bump_alloc(&self, layout: Layout) -> AllocRes<NonNull<[u8]>>;
    fn dealloc(&self, data: NonNull<u8>, layout: Layout);
    fn bump_alloc_zeroed(&self, layout: Layout) -> AllocRes<NonNull<[u8]>> {
        let buf_ptr = self.bump_alloc(layout)?;
        let thin = buf_ptr.as_mut_ptr();

        unsafe {
            thin.write_bytes(0, layout.size());
        }

        Ok(buf_ptr)
    }

    #[allow(clippy::mut_from_ref)]
    fn alloc_val<T>(&self, value: T) -> AllocRes<&mut T> {
        let space = self.bump_alloc(Layout::new::<T>())?;
        let thin = space.as_mut_ptr() as *mut T;
        unsafe { ptr::write(thin, value) };
        Ok(unsafe { &mut *thin })
    }
}

pub(crate) struct ArenaAllocator<B: Buffer<u8>> {
    buf: B,
    next_free: AtomicUsize,
}

impl<B: Buffer<u8>> ArenaAllocatorImpl for ArenaAllocator<B> {
    fn bump_alloc(&self, layout: Layout) -> AllocRes<NonNull<[u8]>> {
        let idx = loop {
            let cur = self.next_free.load(Ordering::Acquire);
            if layout.size() > self.buf.len() - cur {
                return Err(AllocError::with_message(
                    AllocErrorKind::OOM,
                    "Not enough memory in buffer",
                ));
            }

            if let Ok(current) = self.next_free.compare_exchange(
                cur,
                cur + layout.size(),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                break current;
            }
        };

        let buffer = self.buf.as_mut_ptr();
        let buffer = unsafe { buffer.add(idx) };
        let buffer = ptr::slice_from_raw_parts_mut(buffer, layout.size());

        NonNull::new(buffer).ok_or(AllocError::new(AllocErrorKind::InvalidPtr))
    }

    fn dealloc(&self, data: NonNull<u8>, layout: Layout) {
        let cur = self.next_free.load(Ordering::Acquire);
        if layout.size() > cur {
            return;
        }
        let last = cur - layout.size();
        let base_ptr = self.buf.as_ptr();
        let cur_ptr = unsafe { base_ptr.add(last) };
        if cur_ptr == data.as_ptr() {
            // we may try to free the memory, as it seems like the returned object is at the end of the buffer
            _ = self
                .next_free
                .compare_exchange(cur, last, Ordering::AcqRel, Ordering::Relaxed);
        }
    }
}

impl<B: Buffer<u8>> ArenaAllocator<B> {
    pub(crate) fn new_in(buf: B) -> Self {
        Self {
            buf,
            next_free: AtomicUsize::new(0),
        }
    }
}

#[cfg(feature = "alloc")]
mod heap_ {
    use crate::buffer::HeapBuf;

    use super::*;

    #[cfg(feature = "allocator_api")]
    mod alloc_api_ {
        use super::*;

        #[macro_export]
        macro_rules! std_allocator_impl {
            (@impl [$($impl_generics:tt)*] $ty:ty) => {
                unsafe impl<$($impl_generics)*> ::alloc::alloc::Allocator for $ty {
                    fn allocate(&self, layout: ::core::alloc::Layout) -> Result<NonNull<[u8]>, ::alloc::alloc::AllocError> {
                        $crate::ArenaAllocatorImpl::bump_alloc(self, layout).map_err(|e| e.into())
                    }

                    unsafe fn deallocate(&self, ptr: ::core::ptr::NonNull<u8>, layout: ::core::alloc::Layout) {
                        $crate::ArenaAllocatorImpl::dealloc(self, ptr, layout);
                    }

                    fn allocate_zeroed(&self, layout: ::core::alloc::Layout) -> Result<::core::ptr::NonNull<[u8]>, ::alloc::alloc::AllocError> {
                        $crate::ArenaAllocatorImpl::bump_alloc_zeroed(self, layout).map_err(|e| e.into())
                    }
                }
            };

            ($ty:ty) => {
                std_allocator_impl!(@impl [] $ty);
            };

            ($ty:ty where [$($generics:tt)*]) => {
                std_allocator_impl!(@impl [$($generics)*] $ty);
            };
        }

        std_allocator_impl!(HeapAllocator);
        std_allocator_impl!(StackAllocator<N> where [const N: usize]);
    }

    pub struct HeapAllocator(ArenaAllocator<HeapBuf<u8>>);

    impl ArenaAllocatorImpl for HeapAllocator {
        fn bump_alloc(&self, layout: Layout) -> AllocRes<NonNull<[u8]>> {
            ArenaAllocatorImpl::bump_alloc(&self.0, layout)
        }

        fn dealloc(&self, data: NonNull<u8>, layout: Layout) {
            ArenaAllocatorImpl::dealloc(&self.0, data, layout);
        }
    }

    impl HeapAllocator {
        pub fn new(size: usize) -> Self {
            Self(ArenaAllocator::new_in(HeapBuf::new(size)))
        }
    }
}

mod stack_ {
    use crate::buffer::StackBuf;

    use super::*;

    pub struct StackAllocator<const N: usize>(ArenaAllocator<StackBuf<N, u8>>);

    impl<const N: usize> ArenaAllocatorImpl for StackAllocator<N> {
        fn bump_alloc(&self, layout: Layout) -> AllocRes<NonNull<[u8]>> {
            self.0.bump_alloc(layout)
        }

        fn dealloc(&self, data: NonNull<u8>, layout: Layout) {
            self.0.dealloc(data, layout)
        }
    }

    impl<const N: usize> StackAllocator<N> {
        pub fn new() -> Self {
            Self(ArenaAllocator::new_in(StackBuf::new()))
        }
    }

    impl<const N: usize> Default for StackAllocator<N> {
        fn default() -> Self {
            Self::new()
        }
    }
}
