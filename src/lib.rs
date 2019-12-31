//! Bit level reading and writing.
//! 
//! To access [BitReader](./struct.BitReader.html) or `BitWriter` types use `--features std`
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-30

#![cfg_attr(not(feature = "std",), no_std,)]
#![feature(const_fn, const_transmute, never_type, try_trait,)]
#![deny(missing_docs,)]

extern crate core;
extern crate alloc;
#[cfg(test,)]
#[macro_use]
extern crate std;

pub mod bits;
mod bit_read;
mod bit_write;

pub use self::{bits::Bits, bit_read::*, bit_write::*,};

/// The error returned when trying to unwrap an unaligned reader/writer.
#[derive(Debug, Hash,)]
pub struct UnalignedError<R,>(pub(crate) R, pub(crate) Bits,);

impl<R,> UnalignedError<R,> {
  /// Return how far from aligned the reader/writer was.
  #[inline]
  pub const fn misalign(&self,) -> Bits { self.1 }
  /// Unwraps the reader from the error.
  #[inline]
  pub fn into_inner(self,) -> R { self.0 }
}
