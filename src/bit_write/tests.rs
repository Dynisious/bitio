//! Author --- DMorgan  
//! Last Moddified --- 2019-12-29

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
  assert_eq!(writer.write_bits(Bits::B3, 0b101,), Ok(Bits::B3),);
  assert!(writer.write_byte(0b01110110,).is_ok(),);
  assert_eq!(writer.write_bit(true,), Ok(false),);
  assert_eq!(writer.reset(), Some(byte),);
  assert_eq!(writer.write_bits(Bits::B3, 0b110,), Ok(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B5, 0b10111,), Ok(Bits::B5),);
  assert_eq!(writer.into_buffer(), Ok(Some(byte)),);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteSlice() {
  let mut buffer = [0u8; 2];
  let mut writer = WriteSlice::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), 8,);
  assert_eq!(writer.write_bit(true,).ok(), Some(true),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,).ok(), Some(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0111,).ok(), Some(Bits::B4),);
  match writer.into_slice() {
    //The conversion succeeded, return the writer.
    Ok(Some(slice)) => writer = WriteSlice::new(slice,),
    e => panic!("Expected slice, found: {:?}", e,),
  }
  assert_eq!(writer.write_bits(Bits::B4, 0b1010,).ok(), Some(Bits::B4),);
  assert_eq!(writer.to_write(), 4,);
  assert_eq!(writer.write_bits(Bits::B3, 0b001,).ok(), Some(Bits::B3),);
  match writer.into_slice() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    e => panic!("Expected error, found: {:?}", e,),
  }
  assert!(writer.write_byte(!0,).is_ok(),);
  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,), Err(0),);
  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.into_slice(), Ok(None),);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteVec() {
  let mut writer = WriteVec::new();

  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.write_bit(true,).ok(), Some(true),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,).ok(), Some(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0111,).ok(), Some(Bits::B4),);
  if let Err(e) = writer.into_vec() {
    panic!("Expected slice, found: {:?}", e,)
  } else { writer = WriteVec::new() }
  assert_eq!(writer.write_bits(Bits::B4, 0b1010,).ok(), Some(Bits::B4),);
  assert_eq!(writer.to_write(), 4,);
  assert_eq!(writer.write_bits(Bits::B3, 0b001,).ok(), Some(Bits::B3),);
  match writer.into_vec() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    Ok(e) => panic!("Expected error, found: {:?}", e,),
  }
  assert!(writer.write_byte(!0,).is_ok(),);
  assert_eq!(writer.to_write(), 1,);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,), Ok(Bits::B3),);
  assert_eq!(writer.to_write(), 6,);
  assert_eq!(writer.write_bits(Bits::B6, 0,), Ok(Bits::B6),);
  assert_eq!(writer.into_vec().ok(), Some(alloc::vec![0b10100011, 0b11111111, 0b11000000,]),);
}

#[allow(non_snake_case,)]
#[cfg(feature = "std",)]
#[test]
fn test_WriteIO() {
  let mut buffer = [0u8; 2];
  let mut writer = WriteIO::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), 0,);
  assert_eq!(writer.write_bit(true,).ok(), Some(true),);
  assert_eq!(writer.write_bits(Bits::B3, 0b101,).ok(), Some(Bits::B3),);
  assert!(writer.write_byte(0b01111010,).is_ok(),);
  assert_eq!(writer.to_write(), 4,);
  assert_eq!(writer.write_bits(Bits::B3, 0b001,).ok(), Some(Bits::B3),);
  match writer.into_writer() {
    //The conversion failed successfully, return the writer.
    Err(Ok(e)) => writer = e.into_inner(),
    _ => panic!(),
  }
  assert!(writer.write_byte(!0,).is_ok(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,).map_err(|e,| e.0,), Err(Some(Bits::B1)),);
  assert_eq!(writer.to_write(), 0,);
  writer.clear_buffer();
  assert_eq!(writer.into_writer().ok(), Some(&mut [][..]),);
}
