#![no_std]
#![feature(unsafe_cell_access, slice_ptr_get)]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

mod allocator;
#[cfg(feature = "boxed")]
pub mod boxed;
mod buffer;
mod util;

pub use allocator::*;
use thiserror::Error;

pub type AllocRes<T> = Result<T, AllocError>;

#[derive(Error, Debug)]
#[error("AllocError {} occurred\n {:?}", self.kind, self.msg)]
pub struct AllocError {
    kind: AllocErrorKind,
    msg: Option<&'static str>,
}

impl AllocError {
    pub fn new(kind: AllocErrorKind) -> Self {
        Self { kind, msg: None }
    }

    pub fn with_message(kind: AllocErrorKind, msg: &'static str) -> Self {
        Self {
            kind,
            msg: Some(msg),
        }
    }
}

#[cfg(feature = "alloc")]
impl From<AllocError> for alloc::alloc::AllocError {
    fn from(_value: AllocError) -> Self {
        alloc::alloc::AllocError
    }
}

#[derive(Error, Debug)]
pub enum AllocErrorKind {
    #[error("out of memory to allocate")]
    OOM,
    #[error("the passed ptr is invalid")]
    InvalidPtr,
    #[error("Unknown error")]
    Other,
}
