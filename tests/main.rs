#![no_std]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]
#![feature(slice_ptr_get)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod dummy;
#[cfg(feature = "alloc")]
mod heap;
mod stack;

fn main() {}
