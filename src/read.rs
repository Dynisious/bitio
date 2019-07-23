//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-07-24

use std::io::{self, Write, BufRead,};

/// Allows bitwise reads from an inner [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
#[derive(Debug,)]
pub struct BitRead<R,> {
  /// The internal reader.
  inner: R,
  /// The buffer of bits pending reading.
  buffer: u8,
  /// The cursor of which bit is to be read next.
  cursor: u8,
}

impl<R,> BitRead<R,> {
  /// The initial cursor.
  const CURSOR_INIT: u8 = 0x80;

  /// Starts reading in a bitwise fashion.
  #[inline]
  pub const fn new(inner: R,) -> Self {
    Self {
      inner,
      buffer: 0,
      cursor: Self::CURSOR_INIT,
    }
  }
}

impl<R,> BitRead<R,>
  where R: BufRead, {
  /// Advances the internal cursor.
  fn advance_cursor(&mut self,) {
    //Advance the cursor.
    self.cursor >>= 1;

    //If the cursor is aligned, reset the cursor.
    if self.cursor == 0 { self.cursor = Self::CURSOR_INIT }
  }
  /// Reads in the next byte from the reader.
  fn update_buf(&mut self,) -> io::Result<()> {
    //Get the next byte.
    if let Some(byte) = self.inner.fill_buf()?.first() {
      self.buffer = *byte;
    }
    //Consume the byte.
    self.inner.consume(1,);

    Ok(())
  }

  /// Returns the inner buffer.
  /// 
  /// ```rust
  /// use bitio::BitRead;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitRead::new(bytes,);
  /// 
  /// assert_eq!(bits.buffer(), &0,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// assert_eq!(bits.buffer(), &0b0110_0101,);
  /// ```
  #[inline]
  pub const fn buffer(&self,) -> &u8 { &self.buffer }
  /// Returns `true` if the reader is byte aligned.
  /// 
  /// ```rust
  /// use bitio::BitRead;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitRead::new(bytes,);
  /// 
  /// assert_eq!(bits.aligned(), true,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// assert_eq!(bits.aligned(), false,);
  /// 
  /// bits.read_bits(6,);
  /// assert_eq!(bits.aligned(), true,);
  /// ```
  #[inline]
  pub const fn aligned(&self,) -> bool { self.cursor == Self::CURSOR_INIT }
  /// Returns the number of bits which need to be read before the reader is byte aligned.
  /// 
  /// ```rust
  /// use bitio::BitRead;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitRead::new(bytes,);
  /// 
  /// assert_eq!(bits.to_read(), 0,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// assert_eq!(bits.to_read(), 6,);
  /// ```
  pub fn to_read(&self,) -> u32 {
    let zeros = self.cursor.trailing_zeros();

    if zeros == 7 { 0 }
    else { zeros + 1 }
  }
  /// Reads the next bit from the internal reader.
  /// 
  /// ```rust
  /// use bitio::BitRead;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitRead::new(bytes,);
  /// 
  /// assert_eq!(bits.read_bit().ok(), Some(false),);
  /// assert_eq!(bits.read_bit().ok(), Some(true),);
  /// assert_eq!(bits.read_bit().ok(), Some(true),);
  /// ```
  pub fn read_bit(&mut self,) -> io::Result<bool> {
    if self.aligned() { self.update_buf()?; }

    //Read the bit.
    let bit = (self.buffer & self.cursor) != 0;

    //Advance the cursor.
    self.advance_cursor();

    Ok(bit)
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
  /// ```rust
  /// use bitio::BitRead;
  /// 
  /// let bytes = &[0b0110_0101][..];
  /// let mut bits = BitRead::new(bytes,);
  /// 
  /// bits.read_bit();
  /// bits.read_bit();
  /// 
  /// assert_eq!(bits.read_bits(4,).ok(), Some(0b0001_1001),);
  /// ```
  pub fn read_bits(&mut self, bits: u8,) -> io::Result<u8> {
    assert!(0 < bits && bits <= 8, "0 <= `bits` > 8",);
    
    if self.aligned() { self.update_buf()?; }

    let to_read = self.to_read() as u8;

    if bits < to_read {
      //There's enough bits stored in the buffer.

      //Advance the cursor.
      self.cursor >>= bits;
      
      //Get the bits.
      Ok(self.buffer >> (to_read - bits))
    } else {
      //The read will empty the buffer.

      let shift = bits - to_read;

      //Set the cursor as needed.
      self.cursor = Self::CURSOR_INIT >> shift;
      
      //Read out the current buffer.
      let buffer = self.buffer.wrapping_shl(shift as u32,);

      //Update the buffer.
      if shift > 0 { self.update_buf()?; }

      Ok(buffer | (self.buffer >> to_read))
    }
  }
  /// Returns the inner reader.
  pub fn into_inner(self,) -> R { self.inner }
}

impl<W,> Write for BitRead<W,>
  where W: Write, {
  #[inline]
  fn write(&mut self, buf: &[u8],) -> io::Result<usize> {
    self.inner.write(buf,)
  }
  #[inline]
  fn flush(&mut self,) -> io::Result<()> {
    self.inner.flush()
  }
}

#[cfg(test,)]
mod tests {
  use super::*;

  #[test]
  fn test_bit_read() {
    use std::io::Read;

    let bytes = &[0b1001_1101, 0b0110_1001,][..];
    let mut bits = BitRead::new(bytes,);

    assert_eq!(bits.aligned(), true,);
    assert_eq!(bits.read_bit().ok(), Some(true),);
    assert_eq!(bits.read_bit().ok(), Some(false),);
    assert_eq!(bits.read_bits(4,).ok().map(|x,| x & 0x0f,), Some(0b0111),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_read(), 2,);

    let mut bits = bits.into_inner();
    let bytes = Some(bytes[1]);
    let buf = &mut [0; 1];
    let bits = bits.read(buf,).ok()
      .map(|_,| buf[0],);
    assert_eq!(bits, bytes,);
  }
}
