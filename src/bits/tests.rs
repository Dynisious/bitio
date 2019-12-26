//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

#![cfg(test,)]

use super::*;

#[test]
fn test_bits() {
  let bits = (1..=8).map(|b,| unsafe { Bits::from_u8(b,) },);
  
  for bit in bits {
    assert_eq!(8 - bit.mask().leading_zeros(), bit as u32, "`mask` failed on {}", bit,);
    assert_eq!(bit.not_mask().trailing_zeros(), bit as u32, "`not_mask` failed on {}", bit,);
    assert_eq!(Bits::unused_bits(bit.mask(),) as u8, 8 - bit as u8, "`unused_bits` failed on {}", bit,);
    assert_eq!(Bits::used_bits(bit.mask(),), bit, "`used_bits` failed on {}", bit,);
    assert_eq!(bit.bit(), 1 << bit as u8 - 1, "`bit` failed on {}", bit,);
    assert_eq!(Bits::as_u8(Some(bit),), bit as u8, "`as_u8` failed on {}", bit,);
  }
  assert_eq!(Bits::as_u8(None,), 0, "`as_u8` failed on `None`",);
  assert_eq!(Bits::B2 + Bits::B3, Bits::B5, "`Bits + Bits` failed",);
  assert_eq!(Bits::B3 - Bits::B2, Bits::B1, "`Bits - Bits` failed",);
  assert_eq!(Bits::B2 + 8, 10, "`Bits + u8` failed",);
  assert_eq!(Bits::B3 - 3, 0, "`Bits - u8` failed",);
  assert_eq!(Bits::try_from(0,).ok(), None, "`try_from 1` failed",);
  assert_eq!(Bits::try_from(9,).ok(), None, "`try_from 2` failed",);
  assert_eq!(Bits::try_from(5,).ok(), Some(Bits::B5), "`try_from 3` failed",);
}
