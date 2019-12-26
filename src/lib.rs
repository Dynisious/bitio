//! Bit level reading and writing.
//! 
//! To access [BitReader](./struct.BitReader.html) or `BitWriter` types use `--features std`
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

#![cfg_attr(not(feature = "std",), no_std,)]
#![feature(const_fn, const_transmute, never_type, try_trait,)]
#![deny(missing_docs,)]

#[macro_use]
extern crate core;

pub mod bits;
mod bit_read;
mod bit_write;

pub use self::{bit_read::*, bit_write::*,};

/// The error returned when trying to unwrap an unaligned reader.
#[derive(PartialEq, Eq, Debug, Hash,)]
pub struct UnalignedError<R,>(pub(crate) R,);

impl<R,> UnalignedError<R,> {
  /// Unwraps the reader from the error.
  #[inline]
  pub fn into_inner(self,) -> R { self.0 }
}
