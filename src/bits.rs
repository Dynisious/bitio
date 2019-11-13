//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-11-13

use std::{
  cmp::Ordering,
  ops::{
    Deref,
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
    Shl, ShlAssign,
    Shr, ShrAssign,
  },
  convert::{TryFrom, AsRef,},
  borrow::Borrow,
};

/// An enum of the number of possible bits to read at once.
/// 
/// Effectivelly a bounded `[u8]`
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug,)]
pub enum Bits {
  /// 0
  B0 = 0,
  /// 1
  B1 = 1,
  /// 2
  B2 = 2,
  /// 3
  B3 = 3,
  /// 4
  B4 = 4,
  /// 5
  B5 = 5,
  /// 6
  B6 = 6,
  /// 7
  B7 = 7,
  /// 8
  B8 = 8,
}

impl Bits {
  /// The maximum value as a `Bits` value.
  pub const MAX_BITS: Self = Bits::B8;
  /// The maximum value as a `[u8]` value.
  pub const MAX_U8: u8 = Self::MAX_BITS as u8;

  /// Constructs a new `Bits` instance.
  #[inline]
  pub fn new(from: u8,) -> Self {
    use std::convert::TryInto;

    from.try_into().expect(&format!("`u8` value is greater than the maximum of {}", Bits::MAX_U8,),)
  }
}

impl PartialEq<u8> for Bits {
  #[inline]
  fn eq(&self, rhs: &u8,) -> bool { self.as_ref() == rhs }
}

impl PartialOrd<u8> for Bits {
  #[inline]
  fn partial_cmp(&self, rhs: &u8,) -> Option<Ordering> { self.as_ref().partial_cmp(rhs,) }
}

impl PartialEq<Bits> for u8 {
  #[inline]
  fn eq(&self, rhs: &Bits,) -> bool { self == rhs.as_ref() }
}

impl PartialOrd<Bits> for u8 {
  #[inline]
  fn partial_cmp(&self, rhs: &Bits,) -> Option<Ordering> { self.partial_cmp(rhs.as_ref(),) }
}

impl Add<u8> for Bits {
  type Output = u8;

  #[inline]
  fn add(self, rhs: u8,) -> u8 { self as u8 + rhs }
}

impl Sub<u8> for Bits {
  type Output = u8;

  #[inline]
  fn sub(self, rhs: u8,) -> u8 { self as u8 - rhs }
}

impl Mul<u8> for Bits {
  type Output = u8;

  #[inline]
  fn mul(self, rhs: u8,) -> u8 { self as u8 * rhs }
}

impl Div<u8> for Bits {
  type Output = u8;

  #[inline]
  fn div(self, rhs: u8,) -> u8 { self as u8 / rhs }
}

impl Shl<u8> for Bits {
  type Output = u8;

  #[inline]
  fn shl(self, rhs: u8,) -> u8 { (self as u8) << rhs }
}

impl Shr<u8> for Bits {
  type Output = u8;

  #[inline]
  fn shr(self, rhs: u8,) -> u8 { self as u8 >> rhs }
}

impl Add<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn add(mut self, rhs: Bits,) -> u8 { self += rhs; self }
}

impl AddAssign<Bits> for u8 {
  #[inline]
  fn add_assign(&mut self, rhs: Bits,) { *self += rhs as u8 }
}

impl Sub<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn sub(mut self, rhs: Bits,) -> u8 { self -= rhs; self }
}

impl SubAssign<Bits> for u8 {
  #[inline]
  fn sub_assign(&mut self, rhs: Bits,) { *self -= rhs as u8 }
}

impl Mul<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn mul(mut self, rhs: Bits,) -> u8 { self *= rhs; self }
}

impl MulAssign<Bits> for u8 {
  #[inline]
  fn mul_assign(&mut self, rhs: Bits,) { *self *= rhs as u8 }
}

impl Div<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn div(mut self, rhs: Bits,) -> u8 { self /= rhs; self }
}

impl DivAssign<Bits> for u8 {
  #[inline]
  fn div_assign(&mut self, rhs: Bits,) { *self /= rhs as u8 }
}

impl Shl<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn shl(mut self, rhs: Bits,) -> u8 { self <<= rhs; self }
}

impl ShlAssign<Bits> for u8 {
  #[inline]
  fn shl_assign(&mut self, rhs: Bits,) { *self <<= rhs as u8 }
}

impl Shr<Bits> for u8 {
  type Output = u8;

  #[inline]
  fn shr(mut self, rhs: Bits,) -> u8 { self >>= rhs; self }
}

impl ShrAssign<Bits> for u8 {
  #[inline]
  fn shr_assign(&mut self, rhs: Bits,) { *self >>= rhs as u8 }
}

impl TryFrom<u8> for Bits {
  type Error = u8;

  #[inline]
  fn try_from(from: u8,) -> Result<Self, Self::Error> {
    if from > Self::MAX_BITS { Err(from) }
    else { unsafe { Ok(std::mem::transmute(from,)) } }
  }
}

impl Into<u8> for Bits {
  #[inline]
  fn into(self,) -> u8 { self as u8 }
}

impl AsRef<u8> for Bits {
  #[inline]
  fn as_ref(&self,) -> &u8 {
    unsafe { std::mem::transmute(self,) }
  }
}

impl Borrow<u8> for Bits {
  #[inline]
  fn borrow(&self,) -> &u8 {
    unsafe { std::mem::transmute(self,) }
  }
}

impl Deref for Bits {
  type Target = u8;

  #[inline]
  fn deref(&self,) -> &Self::Target {
    unsafe { std::mem::transmute(self,) }
  }
}

#[cfg(test,)]
mod tests {
  use super::*;

  #[test]
  #[allow(non_snake_case,)]
  fn test_Bits_size() {
    assert_eq!(std::mem::size_of::<Bits>(), 1, "`Bits` incorrect size",);
  }
}
