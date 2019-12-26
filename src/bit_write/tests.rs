//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

#![cfg(test,)]

use super::*;

#[allow(non_snake_case,)]
#[test]
fn test_WriteByte() {
  let byte = 0b11010111;
  let mut writer = WriteByte::new();

  assert_eq!(writer.to_write(), 8,);
  assert_eq!(writer.into_buffer(), Ok(None),);
  assert_eq!(writer.write_bit(true,), Ok(true),);
  assert!(writer.into_buffer().is_err(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,), Ok(3),);
  assert_eq!(writer.write_byte(0b01110110,), Ok(4),);
  assert_eq!(writer.write_bit(true,), Ok(false),);
  assert_eq!(writer.reset(), Some(byte),);
  assert_eq!(writer.write_bits(Bits::B3, 0b110,), Ok(3),);
  assert_eq!(writer.write_bits(Bits::B5, 0b10111,), Ok(5),);
  assert_eq!(writer.into_buffer(), Ok(Some(byte)),);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteSlice() {
  let mut buffer = [0u8; 2];
  let mut writer = WriteSlice::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), 8,);
  assert_eq!(writer.write_bit(true,).ok(), Some(true),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,).ok(), Some(3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0111,).ok(), Some(4),);
  match writer.into_slice() {
    //The conversion succeeded, return the writer.
    Ok(Some(slice)) => writer = WriteSlice::new(slice,),
    e => panic!("Expected slice, found: {:?}", e,),
  }
  assert_eq!(writer.write_bits(Bits::B4, 0b1010,).ok(), Some(4),);
  assert_eq!(writer.to_write(), 4,);
  assert_eq!(writer.write_bits(Bits::B3, 0b001,).ok(), Some(3),);
  match writer.into_slice() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    e => panic!("Expected error, found: {:?}", e,),
  }
  assert_eq!(writer.write_byte(!0,).ok(), Some(1),);
  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,), Ok(0),);
  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.into_slice(), Ok(None),);
}

#[allow(non_snake_case,)]
#[cfg(feature = "std",)]
#[test]
fn test_WriteIO() {
  let mut buffer = [0u8; 2];
  let mut writer = WriteIO::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), 8,);
  assert_eq!(writer.write_bit(true,).ok(), Some(true),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,).ok(), Some(3),);
  assert_eq!(writer.write_byte(0b01111010,).ok(), Some(8),);
  assert_eq!(writer.to_write(), 4,);
  assert_eq!(writer.write_bits(Bits::B3, 0b001,).ok(), Some(3),);
  match writer.into_writer() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    _ => panic!(),
  }
  assert_eq!(writer.write_byte(!0,).ok(), Some(8),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,).map_err(|e,| e.0,), Err(1),);
  assert_eq!(writer.to_write(), 0,);
  writer.clear_buffer();
  assert!(writer.into_writer().is_ok(),);
}
