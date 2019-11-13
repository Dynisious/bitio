//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-11-13

use crate::Bits;
use std::iter::{Iterator, Peekable,};

/// An `[Iterator]` over bits instead of bytes.
pub struct BitIter<Iter: Iterator<Item = u8>,> {
  iter: Peekable<Iter>,
  cursor: u8,
}

impl<I,> BitIter<I,>
  where I: Iterator<Item = u8>, {
  /// The initial cursor position.
  const CURSOR_INIT: u8 = 0x80;

  /// Creates a new `BitIter` around the passed `[Iterator]`.
  pub const fn new(iter: Peekable<I>,) -> Self {
    Self { iter, cursor: Self::CURSOR_INIT, }
  }

  /// Gets the byte being read from currently.
  /// 
  /// If no byte was currently being read from the cursor will be reset for reading the
  /// next byte.
  fn buffer(&mut self,) -> Option<u8> {
    //Get the buffer.
    let buffer = self.iter.peek().copied();

    //If the reader is aligned, reset the cursor for reading.
    if buffer.is_some() && self.aligned() { self.cursor = Self::CURSOR_INIT }
    
    buffer
  }
  /// Checks the cursor to determine if the reader has read all of the current byte and
  /// if so consumes the current byte.
  #[inline]
  fn check_cursor(&mut self,) {
    //If the reader is aligned after the read, consume the completed byte.
    if self.cursor == 0 { self.iter.next(); }
  }

  /// Returns `true` if the reader is byte aligned.
  /// 
  /// ```rust
  /// use bitio::{Bits, BitIter,};
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitIter::new(bytes.iter().copied().peekable(),);
  /// 
  /// assert_eq!(bits.aligned(), true,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// assert_eq!(bits.aligned(), false,);
  /// 
  /// bits.read_bits(Bits::B6,);
  /// assert_eq!(bits.aligned(), true,);
  /// ```
  #[inline]
  pub const fn aligned(&self,) -> bool { self.cursor & 0x7F == 0 }
  /// Returns the number of bits which need to be read from the current byte for the
  /// reader to be byte aligned.
  /// 
  /// ```rust
  /// use bitio::BitIter;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitIter::new(bytes.iter().copied().peekable(),);
  /// 
  /// assert_eq!(bits.to_read(), 0,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// assert_eq!(bits.to_read(), 6,);
  /// ```
  pub fn to_read(&self,) -> Bits {
    let zeros = self.cursor.trailing_zeros() as u8;
    
    if dbg!(zeros) >= 7 { Bits::B0 }
    else { unsafe { std::mem::transmute(zeros + 1,) } }
  }
  /// Reads the next bit from the internal reader.
  /// 
  /// ```rust
  /// use bitio::BitIter;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitIter::new(bytes.iter().copied().peekable(),);
  /// 
  /// assert_eq!(bits.read_bit(), Some(false),);
  /// assert_eq!(bits.read_bit(), Some(true),);
  /// assert_eq!(bits.read_bit(), Some(true),);
  /// ```
  pub fn read_bit(&mut self,) -> Option<bool> {
    //Get the buffer byte.
    let buffer = self.buffer()?;

    //Read the bit.
    let bit = (buffer & self.cursor) != 0;

    //Advance the cursor.
    self.cursor = self.cursor.wrapping_shr(1,);

    //Check the cursor after the read.
    self.check_cursor();

    Some(bit)
  }
  /// Reads some bits from the reader.
  /// 
  /// The bits will populate the lower bits of the byte.
  /// 
  /// The higher bits of the byte are not guarenteed to be zeroed.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to read from the range (1..=8).  
  /// 
  /// # Panics
  /// 
  /// If `bits` < 0 or 8 < `bits`
  /// 
  /// ```rust
  /// use bitio::{Bits, BitIter,};
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitIter::new(bytes.iter().copied().peekable(),);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// 
  /// assert_eq!(bits.read_bits(Bits::B4,), Some(0b0001_1001),);
  /// ```
  pub fn read_bits(&mut self, bits: Bits,) -> Option<u8> {
    if bits == 0 { return Some(0) }

    assert!(0 < bits && bits <= 8, "`bits` < 0 or 8 < `bits`",);
    
    //Get the buffer byte.
    let mut buffer = self.buffer()?;
    //Get the number of bits still to be read from this buffer byte.
    let mut to_read = self.to_read() as u8;

    //Move the data into `buffer`.
    if to_read >= bits {
      //There are enough bits in the buffer.

      //Get the number of bits which will remain in the buffer.
      to_read -= bits;

      //Get the bits into the lower bits of the buffer. 
      buffer = buffer.wrapping_shr(to_read as u32,);
      //Advance the cursor as needed.
      self.cursor = self.cursor.wrapping_shr(bits as u32,);

      //Check the cursor.
      self.check_cursor();
    } else {
      //There are not enough bits in the buffer.

      //Get the number of bits which need to be retreived from the next buffer byte.
      let to_read = bits - to_read;

      //Make space for the extra bits.
      buffer <<= to_read;
      self.cursor = 0;

      //Consume the current byte.
      self.iter.next();

      //Read in the new data into the lower bits.
      buffer ^= self.buffer()?.wrapping_shr(8 - to_read as u32,);
      //Advance the cursor for the remaining bits.
      self.cursor = Self::CURSOR_INIT.wrapping_shr(to_read as u32,);
    }

    Some(buffer)
  }
  /// Drops any bytes which need to be read for the reader to be `Byte` alligned.
  pub fn align(&mut self,) -> Option<()> {
    let to_read = self.to_read();

    self.read_bits(to_read,)?;
    Some(())
  }
  /// Returns the inner reader.
  pub fn into_inner(self,) -> Peekable<I> { self.iter }
}

impl<I,> Iterator for BitIter<I,>
  where I: Iterator<Item = u8>, {
  type Item = bool;

  #[inline]
  fn next(&mut self,) -> Option<bool> { self.read_bit() }
  #[inline]
  fn size_hint(&self,) -> (usize, Option<usize>,) {
    let (low, high,) = self.iter.size_hint();

    (low * 8, high.map(|high,| high * 8,),)
  }
}

#[cfg(test,)]
mod tests {
  use super::*;

  #[test]
  #[allow(non_snake_case,)]
  fn test_BitIter() {
    let bytes = &[0b1001_1101, 0b0110_1001,][..];
    let mut bits = BitIter::new(bytes.iter().copied().peekable(),);

    assert_eq!(bits.aligned(), true,);
    assert_eq!(bits.read_bit(), Some(true),);
    assert_eq!(bits.read_bit(), Some(false),);
    assert_eq!(bits.read_bits(Bits::B4,).map(|x,| x & 0x0f,), Some(0b0111),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_read(), Bits::B2,);
    assert_eq!(bits.read_bits(Bits::B4,).map(|x,| x & 0x0f,), Some(0b0101),);

    let bits = bits.into_inner();
    assert_eq!(bits.collect::<Vec<_>>(), &bytes[1..],);

    let mut bits = BitIter::new([0u8; 0].as_ref().iter().copied().peekable(),);
    assert_eq!(bits.read_bits(Bits::B0,), Some(0),);
  }
}
