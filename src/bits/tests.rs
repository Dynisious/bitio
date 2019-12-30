//! Author --- DMorgan  
//! Last Moddified --- 2019-12-30

#![cfg(test,)]

use super::*;

#[test]
fn test_bits() {
  for bit in Bits::BITS.iter().copied() {
    assert_eq!(bit.recip().and_then(Bits::recip,).unwrap_or(Bits::B8,), bit, "`recip` is not reciprocal",);
    assert_eq!(8 - bit.mask().leading_zeros(), bit as u32, "`mask` failed on {}", bit,);
    assert_eq!(bit.not_mask().trailing_zeros(), bit as u32, "`not_mask` failed on {}", bit,);
    assert_eq!(
      Bits::unused_bits(bit.mask(),).and_then(Bits::recip,),
      Some(bit).filter(|&b,| b != Bits::B8,),
      "`unused_bits` failed on {}", bit,
    );
    assert_eq!(Bits::used_bits(bit.mask(),), Some(bit), "`used_bits` failed on {}", bit,);
    assert_eq!(bit.bit(), 1u8.wrapping_shl(bit as u32 - 1,), "`bit` failed on {}", bit,);
    assert_eq!(Bits::as_u8(Some(bit),), bit as u8, "`as_u8` failed on {}", bit,);
  }
  assert_eq!(Bits::as_u8(None,), 0, "`as_u8` failed on `None`",);
  assert_eq!(Bits::try_from(0,), Err(FromU8Error((),)), "`try_from 1` failed",);
  assert_eq!(Bits::try_from(9,), Err(FromU8Error((),)), "`try_from 2` failed",);
  assert_eq!(Bits::try_from(5,), Ok(Bits::B5), "`try_from 3` failed",);
}
