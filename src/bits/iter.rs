//! Defines iterators for `Bits`.
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-30

use super::*;
use core::iter::{Iterator, ExactSizeIterator,};

/// An iterator which cycles through all of the bits from `B1` too `B8`.
#[derive(PartialEq, Eq, Clone, Copy, Debug,)]
pub struct Cycle(pub(crate) Option<Bits>,);

impl Cycle {
  /// A new `Cycle` instace.
  pub const CYCLE: Self = Cycle(Some(Bits::B1),);

  /// Gets the current state of the `Cycle`.
  #[inline]
  pub const fn into(self,) -> Option<Bits> { self.0 }
}

impl Iterator for Cycle {
  type Item = Bits;

  fn size_hint(&self,) -> (usize, Option<usize>,) {
    let len = self.len();

    (len, Some(len),)
  }
  fn next(&mut self,) -> Option<Self::Item> {
    let next = Bits::try_from(self.0? as u8 + 1,).ok();

    core::mem::replace(&mut self.0, next,)
  }
}

impl ExactSizeIterator for Cycle {
  fn len(&self,) -> usize { (8 - Bits::as_u8(self.0,)) as usize }
}

impl Into<Option<Bits>> for Cycle {
  #[inline]
  fn into(self,) -> Option<Bits> { self.0 }
}

/// An iterator which cycles through all of the bits from `B8` too `B1`.
#[derive(PartialEq, Eq, Clone, Copy, Debug,)]
pub struct RevCycle(pub(crate) Option<Bits>,);

impl RevCycle {
  /// A new `RevCycle` instace.
  pub const CYCLE: Self = RevCycle(Some(Bits::B8),);

  /// Gets the current state of the `RevCycle`.
  #[inline]
  pub const fn into(self,) -> Option<Bits> { self.0 }
}

impl Iterator for RevCycle {
  type Item = Bits;

  fn size_hint(&self,) -> (usize, Option<usize>,) {
    let len = self.len();

    (len, Some(len),)
  }
  fn next(&mut self,) -> Option<Self::Item> {
    let next = Bits::try_from(self.0? as u8 - 1,).ok();

    core::mem::replace(&mut self.0, next,)
  }
}

impl ExactSizeIterator for RevCycle {
  #[inline]
  fn len(&self,) -> usize { Bits::as_u8(self.0,) as usize }
}

impl Into<Option<Bits>> for RevCycle {
  #[inline]
  fn into(self,) -> Option<Bits> { self.0 }
}
