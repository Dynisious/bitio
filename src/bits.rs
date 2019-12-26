//! Defines types useful for using individual bits from a byte.
//! 
//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

use core::{
  fmt,
  num::NonZeroU8,
  ops::{
    Deref,
    Add, AddAssign,
    Sub, SubAssign,
    Mul, Div,
  },
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
  /// Creates a bit mask which covers the number of low bits.
  /// 
  /// This is the inverse of `not_mask`.  
  #[inline]
  pub const fn mask(self,) -> u8 { !0 >> (8 - self as u8) }
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
  pub const fn bit(self,) -> u8 { 1 << (self as u8 - 1) }
  /// Converts the `Bits` value into a `u8`.
  #[inline]
  pub fn as_u8(from: Option<Self>,) -> u8 { from.map(Self::into,).unwrap_or(0,) }
}

impl PartialEq<u8> for Bits {
  #[inline]
  fn eq(&self, rhs: &u8,) -> bool {
    u8::eq(self, rhs,)
  }
}

impl PartialOrd<u8> for Bits {
  #[inline]
  fn partial_cmp(&self, rhs: &u8,) -> Option<Ordering> {
    u8::partial_cmp(self, rhs,)
  }
}

impl<T,> Add<T> for Bits
  where T: Borrow<Self>, {
  type Output = Self;

  fn add(mut self, rhs: T,) -> Self::Output {
    self = unsafe { Self::from_u8(self as u8 + *rhs.borrow() as u8,) };

    debug_assert!((self as u8).wrapping_sub(1,) <= 7, "`Bits` value overflowed",);

    self
  }
}

impl<T,> AddAssign<T> for Bits
  where T: Borrow<Self>, {
  #[inline]
  fn add_assign(&mut self, rhs: T,) { *self = *self + *rhs.borrow() }
}

impl<T,> Sub<T> for Bits
  where T: Borrow<Self>, {
  type Output = Self;

  fn sub(mut self, rhs: T,) -> Self::Output {
    self = unsafe { Self::from_u8(self as u8 - *rhs.borrow() as u8,) };

    debug_assert!((self as u8).wrapping_sub(1,) <= 7, "`Bits` value underflowed",);

    self
  }
}

impl<T,> SubAssign<T> for Bits
  where T: Borrow<Self>, {
  #[inline]
  fn sub_assign(&mut self, rhs: T,) { *self = *self - *rhs.borrow() }
}

impl Add<Option<Self>> for Bits {
  type Output = Self;

  #[inline]
  fn add(self, rhs: Option<Self>,) -> Self::Output {
    <Self as Add>::add(self, unsafe { core::mem::transmute(Self::as_u8(rhs,),) },)
  }
}

impl AddAssign<Option<Self>> for Bits {
  #[inline]
  fn add_assign(&mut self, rhs: Option<Self>,) {
    <Self as AddAssign>::add_assign(self, unsafe { core::mem::transmute(Self::as_u8(rhs,),) },)
  }
}

impl Sub<Option<Self>> for Bits {
  type Output = Self;

  #[inline]
  fn sub(self, rhs: Option<Self>,) -> Self::Output {
    <Self as Sub>::sub(self, unsafe { core::mem::transmute(Self::as_u8(rhs,),) },)
  }
}

impl SubAssign<Option<Self>> for Bits {
  #[inline]
  fn sub_assign(&mut self, rhs: Option<Self>,) {
    <Self as SubAssign>::sub_assign(self, unsafe { core::mem::transmute(Self::as_u8(rhs,),) },)
  }
}

impl Add<u8,> for Bits {
  type Output = u8;

  #[inline]
  fn add(self, rhs: u8,) -> Self::Output { self as u8 + rhs }
}

impl Sub<u8,> for Bits {
  type Output = u8;

  #[inline]
  fn sub(self, rhs: u8,) -> Self::Output { self as u8 - rhs }
}

impl Mul<u8,> for Bits {
  type Output = u8;

  #[inline]
  fn mul(self, rhs: u8,) -> Self::Output { self as u8 * rhs }
}

impl Div<u8,> for Bits {
  type Output = u8;

  #[inline]
  fn div(self, rhs: u8,) -> Self::Output { self as u8 / rhs }
}

impl Deref for Bits {
  type Target = u8;

  #[inline]
  fn deref(&self,) -> &Self::Target { unsafe { core::mem::transmute(self,) } }
}

impl AsRef<u8> for Bits {
  #[inline]
  fn as_ref(&self,) -> &u8 { &**self }
}

impl Borrow<u8> for Bits {
  #[inline]
  fn borrow(&self,) -> &u8 { &**self }
}

impl Into<u8> for Bits {
  #[inline]
  fn into(self,) -> u8 { self as u8 }
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
