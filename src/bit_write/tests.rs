//! Author --- DMorgan  
//! Last Moddified --- 2019-12-31

#![cfg(test,)]

use super::*;

#[allow(non_snake_case,)]
#[test]
fn test_WriteByte() {
  let byte = 0b11010111;
  let mut writer = WriteByte::EMPTY;

  assert_eq!(writer.to_write(), Some(Bits::B8),);
  assert_eq!(writer.into_buffer().ok(), Some(0),);
  assert!(writer.write_bit(true,).is_ok(),);
  assert!(writer.into_buffer().is_err(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b110101,), Ok(Bits::B3),);
  assert_eq!(writer.write_byte(0b01110110,).err(), Some(Bits::B4),);
  assert_eq!(writer.write_bit(true,).err(), Some(Bits::B1),);
  assert_eq!(writer.reset(), byte,);
  assert_eq!(writer.write_bits(Bits::B3, 0b100110,), Ok(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B5, 0b11110111,), Ok(Bits::B5),);
  assert_eq!(writer.into_buffer().ok(), Some(byte),);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteSlice() {
  let mut buffer = [0u8; 2];
  let mut writer = WriteSlice::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), None,);
  assert!(writer.write_bit(true,).is_ok(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111101,).ok(), Some(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0110111,).ok(), Some(Bits::B4),);
  match writer.into_slice() {
    //The conversion succeeded, return the writer.
    Ok(Some(slice)) => assert_eq!(slice, &[0,],),
    e => panic!("Expected slice, found: {:?}", e,),
  }

  for b in buffer.iter_mut() { *b = 0 }
  writer = WriteSlice::new(&mut buffer,);
  assert_eq!(writer.write_bits(Bits::B4, 0b1011010,).ok(), Some(Bits::B4),);
  assert_eq!(writer.to_write(), Some(Bits::B4),);
  assert_eq!(writer.write_bits(Bits::B3, 0b10001,).ok(), Some(Bits::B3),);
  match writer.into_slice() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    e => panic!("Expected error, found: {:?}", e,),
  }
  assert!(dbg!(&mut writer).write_byte(!0,).is_ok(),);
  assert_eq!(writer.to_write(), Some(Bits::B1),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111,), Err(Bits::B2),);
  assert_eq!(writer.to_write(), None,);
  assert_eq!(writer.write_byte(0,).err(), Some(Bits::B8),);
  assert_eq!(writer.into_slice().ok(), Some(None),);
  assert_eq!(buffer, [0b10100011, 0b11111111,],);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteVec() {
  let mut writer = WriteVec::new();

  assert_eq!(writer.to_write(), None,);
  assert!(writer.write_bit(true,).is_ok(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111101,).ok(), Some(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0110111,).ok(), Some(Bits::B4),);
  match writer.into_vec() {
    //The conversion succeeded, return the writer.
    Ok(vec) => assert_eq!(vec, &[0b11010111,],),
    Err(e) => panic!("Expected slice, found: {:?}", e,),
  }

  writer = WriteVec::new();
  assert_eq!(writer.write_bits(Bits::B4, 0b1011010,).ok(), Some(Bits::B4),);
  assert_eq!(writer.to_write(), Some(Bits::B4),);
  assert_eq!(writer.write_bits(Bits::B3, 0b10001,).ok(), Some(Bits::B3),);
  match writer.into_vec() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    Ok(e) => panic!("Expected error, found: {:?}", e,),
  }
  assert!(writer.write_byte(!0,).is_ok(),);
  assert_eq!(writer.to_write(), Some(Bits::B1),);
  assert_eq!(writer.write_bits(Bits::B1, 0b111,), Ok(Bits::B1),);
  assert_eq!(writer.to_write(), None,);
  writer.write_byte(0,).expect("Error writing",);
  assert_eq!(writer.into_vec().ok(), Some(alloc::vec![0b10100011, 0b11111111, 0b00000000,],),);
}

#[allow(non_snake_case,)]
#[test]
fn test_WriteIO() {
  #![cfg(feature = "std",)]

  let mut buffer = [0u8; 2];
  let mut writer = WriteIO::new(buffer.as_mut(),);

  assert_eq!(writer.to_write(), None,);
  assert!(writer.write_bit(true,).is_ok(),);
  assert_eq!(writer.write_bits(Bits::B3, 0b111101,).ok(), Some(Bits::B3),);
  assert_eq!(writer.write_bits(Bits::B4, 0b0110111,).ok(), Some(Bits::B4),);
  match writer.into_writer() {
    //The conversion succeeded, return the writer.
    Ok(slice) => assert_eq!(slice, &[0,],),
    Err(e) => panic!("Expected slice, found: {:?}", e,),
  }

  for b in buffer.iter_mut() { *b = 0 }
  writer = WriteIO::new(buffer.as_mut(),);
  assert_eq!(writer.write_bits(Bits::B4, 0b1011010,).ok(), Some(Bits::B4),);
  assert_eq!(writer.to_write(), Some(Bits::B4),);
  assert_eq!(writer.write_bits(Bits::B3, 0b10001,).ok(), Some(Bits::B3),);
  match writer.into_writer() {
    //The conversion failed successfully, return the writer.
    Err(e) => writer = e.into_inner(),
    Ok(e) => panic!("Expected error, found: {:?}", e,),
  }
  assert!(writer.write_byte(!0,).is_ok(),);
  assert_eq!(writer.to_write(), Some(Bits::B1),);
  assert_eq!(writer.write_bits(Bits::B1, 0b111,).ok(), Some(Bits::B1),);
  assert_eq!(writer.to_write(), None,);
  assert!(writer.write_byte(0,).is_err(),);
  let WriteByte { buffer: byte, cursor, } = writer.clear_buffer();
  assert_eq!((byte, cursor,), (0, None,),);
  match writer.into_writer() {
    Ok(slice) => assert_eq!(slice, [].as_mut(),),
    Err(e) => panic!("Expected slice, found: {:?}", e,),
  }
  assert_eq!(buffer, [0b10100011u8, 0b11111111,],);
}
