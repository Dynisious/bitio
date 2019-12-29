//! Defines types useful for using individual bits from a byte.
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-29

use core::{
  fmt,
  num::NonZeroU8,
  ops::Deref,
  cmp::Ordering,
  borrow::Borrow,
  convert::{AsRef, TryFrom, Infallible,},
};

mod tests;

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
  #[inline]
  pub const fn recip(self,) -> Self { unsafe { Bits::from_u8(8 - self as u8,) } }
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
  #[inline]
  pub const fn unused_bits(byte: u8,) -> Self {
    unsafe { Self::from_u8(byte.leading_zeros() as u8,) }
  }
  /// Returns the occupied low bits of `byte`.
  /// 
  /// This is the inverse of `unused_bits`.  
  #[inline]
  pub const fn used_bits(byte: u8,) -> Self {
    unsafe { Self::from_u8(Self::B8 as u8 - Self::unused_bits(byte,) as u8,) }
  }
  /// Converts a `u8` into a `Bits` value.
  #[inline]
  pub const unsafe fn from_u8(byte: u8,) -> Self { core::mem::transmute(byte,) }
  /// Creates a bit mask which covers exactly this bit.
  #[inline]
  pub const fn bit(self,) -> u8 { 1u8.wrapping_shl(self as u32 - 1,) }
  /// Converts the `Bits` value into a `u8`.
  #[inline]
  pub fn as_u8(from: Option<Self>,) -> u8 { from.map(|b,| b as u8,).unwrap_or(0,) }
}

impl PartialEq<u8> for Bits {
  #[inline]
  fn eq(&self, rhs: &u8,) -> bool { u8::eq(&self.get(), rhs,) }
}

impl PartialOrd<u8> for Bits {
  #[inline]
  fn partial_cmp(&self, rhs: &u8,) -> Option<Ordering> { u8::partial_cmp(&self.get(), rhs,) }
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash,)]
pub struct FromU8Error(pub(crate) (),);

impl From<!> for FromU8Error {
  #[inline]
  fn from(_: !,) -> Self { FromU8Error((),) }
}

impl From<Infallible> for FromU8Error {
  #[inline]
  fn from(_: Infallible,) -> Self { FromU8Error((),) }
}
