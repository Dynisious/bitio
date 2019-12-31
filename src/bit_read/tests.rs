//! Author --- DMorgan  
//! Last Moddified --- 2019-12-31

#![cfg(test,)]

use super::*;
use alloc::vec::Vec;
use core::iter::FromIterator;

#[allow(non_snake_case,)]
#[test]
fn test_ReadByte() {
  let byte = 0b11010111;
  let mut reader = ReadByte::new(byte,);

  assert_eq!(reader.to_read(), Some(Bits::B8),);
  assert_eq!(reader.read_bit(), Ok(true),);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b1101),);
  assert_eq!(reader.read_bit(), Ok(false),);
  assert_eq!(reader.set(byte,).read_byte(), Ok(byte),);
  assert_eq!(reader.set(byte,).skip(Bits::B3,).expect("Error skipping bits 1",).to_read(), Some(Bits::B5),);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b110101),);
  assert_eq!(reader.set(byte,).skip(Bits::B3,).expect("Error skipping bits 2").read_bits(Bits::B8,), Err(Some(Bits::B5)),);
  assert_eq!(reader.into_buffer().map_err(|b,| b.1,), Err(Bits::B5),);
  assert_eq!(reader.set(byte,).skip(Bits::B8,).expect("Error skipping bits 3").read_bits(Bits::B8,), Err(None),);
  assert_eq!(reader.to_read(), None,);
  assert_eq!(reader.into_buffer().ok(), Some(215),);
  assert_eq!(reader.set(byte,).into_buffer().ok(), Some(byte),);
}

#[allow(non_snake_case,)]
#[test]
fn test_ReadIter() {
  let byte = 0b11010111;
  let bytes = [0b11010110, 0b10111110, 0b01011111,];
  let mut reader = ReadIter::new(bytes.iter(),);

  assert_eq!(reader.to_read(), None,);
  assert_eq!(reader.read_bit(), Ok(true),);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b1101),);
  assert_eq!(reader.read_bit(), Ok(false),);
  assert_eq!(reader.read_byte(), Ok(byte),);
  assert_eq!(reader.skip(Bits::B3,).expect("Error skipping bits 1",).to_read(), None,);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b11110010),);
  assert_eq!(reader.skip(Bits::B3,).expect("Error skipping bits 2").read_bits(Bits::B8,), Err(Some(Bits::B2)),);
  match reader.into_iter() {
    Err(UnalignedError(_, misalign,)) => assert_eq!(misalign, Some(Bits::B2),),
    Ok(_) => panic!("Reader unwrapped unexpectedly",),
  }

  let mut reader = ReadIter::new(core::iter::empty::<u8>(),);
  assert_eq!(reader.read_bits(Bits::B8,), Err(None),);
  assert_eq!(reader.to_read(), None,);
  assert_eq!(reader.into_iter().ok().map(Vec::from_iter,), Some(Vec::new()),);
  assert_eq!(ReadIter::new(bytes.iter(),).into_iter().ok().map(Vec::from_iter,), Some(bytes.iter().collect()),);
}

#[allow(non_snake_case,)]
#[test]
fn test_ReadIO() {
  #![cfg(feature = "std",)]

  let byte = 0b11010111u8;
  let bytes = [0b11010110u8, 0b10111110, 0b01011111,];
  let mut reader = ReadIO::new(bytes.as_ref(),);

  assert_eq!(reader.to_read(), None,);
  assert_eq!(reader.read_bit().ok(), Some(true),);
  assert_eq!(reader.read_bits(Bits::B3,).ok(), Some(0b1101),);
  assert_eq!(reader.read_bit().ok(), Some(false),);
  assert_eq!(reader.read_byte().ok(), Some(byte),);
  assert_eq!(reader.skip(Bits::B3,).expect("Error skipping bits 1",).to_read(), None,);
  assert_eq!(reader.read_bits(Bits::B3,).ok(), Some(0b11110010),);
  assert_eq!(reader.skip(Bits::B3,).expect("Error skipping bits 2").read_bits(Bits::B8,).map_err(|e,| e.0,), Err(Some(Bits::B2)),);
  assert_eq!(reader.into_reader().err().map(|b,| b.1,), Some(Bits::B2),);
  reader.skip(Bits::B2,).expect("Error skipping bits 3",);
  assert_eq!(reader.read_bits(Bits::B8,).map_err(|e,| e.0,), Err(None),);
  assert_eq!(reader.to_read(), None,);
  assert_eq!(reader.into_reader().ok(), Some([].as_ref()),);
  assert_eq!(ReadIO::new(bytes.as_ref(),).into_reader().ok(), Some(bytes.as_ref()),);
}

