//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

#![cfg(test,)]

use super::*;

#[allow(non_snake_case,)]
#[test]
fn test_ReadByte() {
  let byte = 0b11010111;
  let mut reader = ReadByte::new(byte,);

  assert_eq!(reader.to_read(), 8,);
  assert_eq!(reader.read_bit(), Ok(true),);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b1101),);
  assert_eq!(reader.set(byte,).read_byte(), Ok(byte),);
  assert_eq!(reader.set(byte,).skip(Bits::B3,).to_read(), 5,);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b110101),);
  assert_eq!(reader.set(byte,).skip(Bits::B3,).read_bits(Bits::B8,), Err(5),);
  assert!(reader.into_buffer().is_err(),);
  assert_eq!(reader.set(byte,).skip(Bits::B8,).read_bits(Bits::B8,), Err(0),);
  assert_eq!(reader.to_read(), 0,);
  assert_eq!(reader.into_buffer(), Ok(None),);
  assert_eq!(reader.set(byte,).into_buffer(), Ok(Some(byte)),);
}

#[allow(non_snake_case,)]
#[test]
fn test_ReadIter() {
  let bytes = [0b11010111, 0b10100011, 0b00110101,];
  let mut reader = ReadIter::new(bytes.iter().copied(),);

  assert_eq!(reader.to_read(), 0,);
  assert_eq!(reader.read_bit(), Ok(true),);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b1101),);
  assert_eq!(reader.read_byte(), Ok(0b01111010),);
  assert_eq!(reader.skip(Bits::B3,).to_read(), 1,);
  assert_eq!(reader.read_bits(Bits::B3,), Ok(0b010001100),);
  assert_eq!(reader.skip(Bits::B2,).read_bits(Bits::B8,), Err(4),);
  assert!(reader.clone().into_iter().is_err(),);
  assert_eq!(reader.skip(Bits::B4,).read_bits(Bits::B8,), Err(0),);
  assert_eq!(reader.to_read(), 0,);
  assert!(reader.into_iter().is_ok(),);
}

#[allow(non_snake_case,)]
#[cfg(feature = "std",)]
#[test]
fn test_ReadIO() {
  let bytes = [0b11010111, 0b10100011, 0b00110101,];
  let mut reader = ReadIO::new(bytes.as_ref(),);

  assert_eq!(reader.to_read(), 0,);
  assert_eq!(reader.read_bit().ok(), Some(true),);
  assert_eq!(reader.read_bits(Bits::B3,).ok(), Some(0b1101),);
  assert_eq!(reader.read_byte().ok(), Some(0b01111010),);
  assert!(reader.skip(Bits::B3,).is_ok(),);
  assert_eq!(reader.to_read(), 1,);
  assert_eq!(reader.read_bits(Bits::B3,).ok(), Some(0b010001100),);
  assert!(reader.skip(Bits::B2,).is_ok(),);
  assert_eq!(reader.read_bits(Bits::B8,).map_err(|e,| e.0,), Err(4),);
  assert!(reader.clone().into_reader().is_err(),);
  assert!(reader.skip(Bits::B4,).is_ok(),);
  assert_eq!(reader.read_bits(Bits::B8,).map_err(|e,| e.0,), Err(0),);
  assert_eq!(reader.to_read(), 0,);
  assert!(reader.into_reader().is_ok(),);
}
