//! Defines types useful for using individual bits from a byte.
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-30

use core::{
  fmt,
  num::NonZeroU8,
  ops::Deref,
  cmp::Ordering,
  borrow::Borrow,
  convert::{AsRef, TryFrom, Infallible,},
};

mod iter;
mod tests;

pub use self::iter::*;

/// The bits different number of bits making up a byte.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash,)]
pub enum Bits {
  /// 1
  B1 = 1,
  /// 2
  B2,
  /// 3
  B3,
  /// 4
  B4,
  /// 5
  B5,
  /// 6
  B6,
  /// 7
  B7,
  /// 8
  B8,
}

impl Bits {
  /// All of the bits in ascending order.
  pub const BITS: [Bits; 8] = [
    Bits::B1, Bits::B2, Bits::B3, Bits::B4,
    Bits::B5, Bits::B6, Bits::B7, Bits::B8,
  ];

  /// Returns the reciprocal value of `self`.
  /// 
  /// That is the `Bits` value such that `self + self.recip() == 8`.
  pub fn recip(self,) -> Option<Self> { Bits::try_from(8 - self as u8,).ok() }
  /// Creates a bit mask which covers the number of low bits.
  /// 
  /// This is the inverse of `not_mask`.  
  #[inline]
  pub const fn mask(self,) -> u8 { (!0u8).wrapping_shr(8 - self as u32,) }
  /// Creates a bit mask which covers everything but the number of low bits.
  /// 
  /// This is the inverse of `mask`.  
  #[inline]
  pub const fn not_mask(self,) -> u8 { !self.mask() }
  /// Returns the leading zeros of `byte`.
  /// 
  /// This is the inverse of `used_bits`.  
  pub fn unused_bits(byte: u8,) -> Option<Self> {
    Self::try_from(byte.leading_zeros() as u8,).ok()
  }
  /// Returns the occupied low bits of `byte`.
  /// 
  /// This is the inverse of `unused_bits`.  
  pub fn used_bits(byte: u8,) -> Option<Self> {
    Self::try_from(8 - byte.leading_zeros() as u8,).ok()
  }
  /// Converts a `u8` into a `Bits` value.
  #[inline]
  pub const unsafe fn from_u8(byte: u8,) -> Self { core::mem::transmute(byte,) }
  /// Creates a bit mask which covers exactly this bit.
  #[inline]
  pub const fn bit(self,) -> u8 { 1u8.wrapping_shl(self as u32 - 1,) }
  /// Converts the `Bits` value into a `u8`.
  pub fn as_u8(from: Option<Self>,) -> u8 { from.map(|b,| b as u8,).unwrap_or(0,) }
  /// Returns the smaller of the two values.
  #[inline]
  pub fn min(self, rhs: Self,) -> Self {
    unsafe { core::mem::transmute(u8::min(self as u8, rhs as u8,),) }
  }
  /// Returns the larger of the two values.
  #[inline]
  pub fn max(self, rhs: Self,) -> Self {
    unsafe { core::mem::transmute(u8::max(self as u8, rhs as u8,),) }
  }
}

impl PartialEq<u8> for Bits {
  #[inline]
  fn eq(&self, rhs: &u8,) -> bool { u8::eq(&self.get(), rhs,) }
}

impl PartialEq<Option<Bits>> for Bits {
  fn eq(&self, rhs: &Option<Bits>,) -> bool { u8::eq(&self.get(), &Bits::as_u8(*rhs,),) }
}

impl PartialOrd<u8> for Bits {
  #[inline]
  fn partial_cmp(&self, rhs: &u8,) -> Option<Ordering> { u8::partial_cmp(&self.get(), rhs,) }
}

impl PartialOrd<Option<Bits>> for Bits {
  fn partial_cmp(&self, rhs: &Option<Bits>,) -> Option<Ordering> { u8::partial_cmp(&self.get(), &Bits::as_u8(*rhs,),) }
}

impl Deref for Bits {
  type Target = NonZeroU8;

  #[inline]
  fn deref(&self,) -> &Self::Target { unsafe { core::mem::transmute(self,) } }
}

impl AsRef<NonZeroU8> for Bits {
  #[inline]
  fn as_ref(&self,) -> &NonZeroU8 { &*self }
}

impl Borrow<NonZeroU8> for Bits {
  #[inline]
  fn borrow(&self,) -> &NonZeroU8 { &*self }
}

impl Into<NonZeroU8> for Bits {
  #[inline]
  fn into(self,) -> NonZeroU8 { unsafe { core::mem::transmute(self,) } }
}

impl TryFrom<u8> for Bits {
  type Error = FromU8Error;

  fn try_from(from: u8,) -> Result<Self, Self::Error> {
    if from.wrapping_sub(1,) > 7 { return Err(FromU8Error((),)) }

    Ok(unsafe { Self::from_u8(from,) })
  }
}

impl fmt::Display for Bits {
  fn fmt(&self, fmt: &mut fmt::Formatter,) -> fmt::Result { write!(fmt, "B{}", *self as u8,) }
}

impl fmt::Debug for Bits {
  #[inline]
  fn fmt(&self, fmt: &mut fmt::Formatter,) -> fmt::Result { fmt::Display::fmt(self, fmt,) }
}

/// Error when a `u8` cannot be converted to a [Bits] value.
#[derive(PartialEq, Eq, Clone, Copy, Debug,)]
pub struct FromU8Error(pub(crate) (),);

impl From<!> for FromU8Error {
  #[inline]
  fn from(_: !,) -> Self { FromU8Error((),) }
}

impl From<Infallible> for FromU8Error {
  #[inline]
  fn from(_: Infallible,) -> Self { FromU8Error((),) }
}
