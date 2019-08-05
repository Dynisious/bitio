//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-07-24

use std::io::{self, Write, BufRead,};

/// Allows bitwise reads from an inner [`BufRead`](https://doc.rust-lang.org/std/io/trait.BufRead.html).
#[derive(Debug,)]
pub struct BitRead<R,> {
  /// The internal reader.
  inner: R,
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
      cursor: 0,
    }
  }
}

impl<R,> BitRead<R,>
  where R: BufRead, {
  /// Gets the byte being read from currently.
  /// 
  /// If no byte was currently being read from the cursor will be reset for reading the
  /// next byte.
  fn buffer(&mut self,) -> io::Result<u8> {
    //Get the buffer.
    let buffer = self.inner.fill_buf()?;

    match buffer.get(0,).copied() {
      //Get the buffer.
      Some(buffer) => {
        //If the reader is aligned, reset the cursor for reading.
        if self.aligned() { self.cursor = Self::CURSOR_INIT }

        Ok(buffer)
      },
      //EOF reached.
      None => Err(io::ErrorKind::UnexpectedEof.into()),
    }
  }
  /// Checks the cursor to determine if the reader is properly aligned and consumes the
  /// current byte if it is.
  #[inline]
  fn check_cursor(&mut self,) {
    //If the reader is aligned after the read, consume the completed byte.
    if self.aligned() { self.inner.consume(1,); }
  }

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
  pub const fn aligned(&self,) -> bool { self.cursor == 0 }
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
  pub fn to_read(&self,) -> u8 {
    let zeros = self.cursor.trailing_zeros() as u8;

    if zeros == 8 { 0 }
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
    //Get the buffer byte.
    let buffer = self.buffer()?;

    //Read the bit.
    let bit = (buffer & self.cursor) != 0;

    //Advance the cursor.
    self.cursor = self.cursor.wrapping_shr(1,);

    //Check the cursor after the read.
    self.check_cursor();

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
    
    //Get the buffer byte.
    let mut buffer = self.buffer()?;
    //Get the number of bits still to be read from this buffer byte.
    let mut to_read = self.to_read();

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
      self.inner.consume(1,);

      //Read in the new data into the lower bits.
      buffer ^= self.buffer()?.wrapping_shr(8 - to_read as u32,);
      //Advance the cursor for the remaining bits.
      self.cursor = Self::CURSOR_INIT.wrapping_shr(to_read as u32,);
    }

    Ok(buffer)
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
    let bytes = &[0b1001_1101, 0b0110_1001,][..];
    let mut bits = BitRead::new(bytes,);

    assert_eq!(bits.aligned(), true,);
    assert_eq!(bits.read_bit().ok(), Some(true),);
    assert_eq!(bits.read_bit().ok(), Some(false),);
    assert_eq!(bits.read_bits(4,).ok().map(|x,| x & 0x0f,), Some(0b0111),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_read(), 2,);
    assert_eq!(bits.read_bits(4,).ok().map(|x,| x & 0x0f,), Some(0b0101),);

    let bits = bits.into_inner();
    assert_eq!(bits, &bytes[1..],);
  }
}
