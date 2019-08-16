//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-08-16

use std::{
  fmt,
  io::{self, Read, Write, Error,},
};

/// Allows bitwise writing to an inner [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
#[derive(Debug,)]
pub struct BitWrite<W,> {
  /// The internal writer.
  inner: W,
  /// The buffer of bits pending writing.
  buffer: u8,
  /// The cursor of which bit is to be written next.
  cursor: u8,
}

impl<W,> BitWrite<W,> {
  /// The initial cursor.
  const CURSOR_INIT: u8 = 0x80;

  /// Starts writing in a bitwise fashion.
  #[inline]
  pub const fn new(inner: W,) -> Self {
    Self {
      inner,
      buffer: 0,
      cursor: Self::CURSOR_INIT,
    }
  }
}

impl<W,> BitWrite<W,>
  where W: Write, {
  /// Writes the internal buffer as is to the inner writer.
  #[inline]
  fn flush_buf(&mut self,) -> io::Result<()> {
    use std::slice;

    //Flush the buffer.
    self.inner.write_all(slice::from_ref(&self.buffer,),)?;

    //Clear the buffer.
    self.buffer = 0;

    Ok(())
  }
  /// Checks if the buffer has been filled and if so, flushes and resets the writer state
  /// for the next byte.
  fn check_cursor(&mut self,) -> io::Result<()> {
    //Check if the buffer is filled.
    if self.cursor == 0 {
      //Flush the buffer.
      self.flush_buf()?;
      //Reset the cursor for writing.
      self.cursor = Self::CURSOR_INIT;
    }

    Ok(())
  }

  /// Returns `true` if the writer is byte aligned.
  /// 
  /// ```rust
  /// use bitio::BitWrite;
  /// 
  /// let bytes = &mut [0][..];
  /// let mut bits = BitWrite::new(bytes,);
  /// 
  /// assert_eq!(bits.aligned(), true,);
  /// 
  /// bits.write_bit(true,);
  /// bits.write_bit(false,);
  /// assert_eq!(bits.aligned(), false,);
  /// 
  /// bits.write_bits(0, 6,);
  /// assert_eq!(bits.aligned(), true,);
  /// ```
  #[inline]
  pub const fn aligned(&self,) -> bool { self.cursor == Self::CURSOR_INIT }
  /// Returns the number of bits which need to be written before the writer is byte aligned.
  /// 
  /// ```rust
  /// use bitio::BitWrite;
  /// 
  /// let bytes = &mut [0][..];
  /// let mut bits = BitWrite::new(bytes,);
  /// 
  /// assert_eq!(bits.to_write(), 0,);
  /// 
  /// bits.write_bit(true,);
  /// bits.write_bit(false,);
  /// assert_eq!(bits.to_write(), 6,);
  /// ```
  pub fn to_write(&self,) -> u8 {
    let zeros = self.cursor.trailing_zeros() as u8;

    if zeros == 7 { 0 }
    else { zeros + 1 }
  }
  /// Writes the a bit to the internal writer.
  /// 
  /// ```rust
  /// use bitio::BitWrite;
  /// 
  /// let bytes = &mut [0][..];
  /// let mut bits = BitWrite::new(&mut bytes[..],);
  /// 
  /// assert!(bits.write_bit(false,).is_ok(),);
  /// assert!(bits.write_bit(true,).is_ok(),);
  /// assert!(bits.write_bit(true,).is_ok(),);
  /// 
  /// bits.into_inner();
  /// 
  /// assert_eq!(bytes, &[0b0110_0000,],);
  /// ```
  pub fn write_bit(&mut self, bit: bool,) -> io::Result<()> {
    //Read the bit.
    if bit { self.buffer |= self.cursor; }

    //Advance the cursor.
    self.cursor >>= 1;

    //Check the cursor for alignment after the right.
    self.check_cursor()
  }
  /// Writes the bits in `buffer` to the internal writer.
  /// 
  /// # Param
  /// 
  /// buffer --- The buffer of bits, stored in the lower bits.  
  /// bits --- The number of bits from `buffer` to write.  
  /// 
  /// # Panics
  /// 
  /// If `bits` < 0 or 8 < `bits`
  /// 
  /// ```rust
  /// use bitio::BitWrite;
  /// 
  /// let bytes = &mut [0][..];
  /// let mut bits = BitWrite::new(&mut bytes[..],);
  /// 
  /// assert!(bits.write_bit(false,).is_ok(),);
  /// assert!(bits.write_bit(true,).is_ok(),);
  /// assert!(bits.write_bits(0b0111, 4,).is_ok(),);
  /// 
  /// bits.into_inner();
  /// 
  /// assert_eq!(bytes, &[0b0101_1100,],);
  /// ```
  pub fn write_bits(&mut self, mut buffer: u8, bits: u8,) -> io::Result<()> {
    if bits == 0 { return Ok(()) }

    assert!(0 < bits && bits <= 8, "`bits` < 0 or 8 < `bits`",);

    //Zero out the upper bits of `buffer`.
    buffer &= !0 >> (8 - bits);

    let mut to_write = self.to_write() as u8;

    //If we are byte aligned we have 8 bytes to write.
    if to_write == 0 { to_write = 8 }

    //Write the bits in `buffer`.
    if to_write >= bits {
      //There is enough space in the buffer for the bits.

      //Get the number of bits left to be written in the buffer.
      to_write -= bits;

      //Align the bits in `buffer` with the bits to be written into the internal buffer.
      buffer <<= to_write;
      //Write only the bits into the internal buffer.
      self.buffer |= buffer;

      //Advance the cursor as necessary.
      self.cursor >>= bits;

      //Check the cursor for alignment after the right.
      self.check_cursor()
    } else {
      //There is not enough space in the buffer to store the bits.

      //Get the number of bits which will need to be written into the next buffer.
      to_write = bits - to_write;

      //Write in the first bits.
      self.buffer |= buffer >> to_write;
      self.cursor = 0;

      //Flush the internal buffer.
      self.flush_buf()?;

      //Write the remaining bits into the internal buffer.
      self.buffer = buffer << (8 - to_write);
      self.cursor = Self::CURSOR_INIT >> to_write;

      Ok(())
    }
  }
  /// Returns the inner writer flushing any pending bits in the buffer right padded with
  /// zeros.
  pub fn into_inner(mut self,) -> Result<W, IntoInnerError<W>> {
    //If the writer is not aligned flush the buffer.
    if !self.aligned() {
      //Flush the buffer.
      if let Err(error) = self.flush_buf() {
        return Err(IntoInnerError {
          error,
          inner: self.inner,
        })
      }
    }

    Ok(self.inner)
  }
}

impl<R,> Read for BitWrite<R,>
  where R: Read, {
  #[inline]
  fn read(&mut self, buf: &mut [u8],) -> io::Result<usize> {
    self.inner.read(buf,)
  }
}

/// An error returned if there is an error flushing the internal buffer when calling
/// `into_inner` on a [`BitWrite`].
#[derive(Debug,)]
pub struct IntoInnerError<W,> {
  inner: W,
  error: Error,
}

impl<W,> IntoInnerError<W,> {
  /// Returns the inner writer.
  pub fn into_inner(self,) -> W { self.inner }
}

impl<W,> Into<Error> for IntoInnerError<W,> {
  #[inline]
  fn into(self,) -> Error { self.error }
}

impl<W,> fmt::Display for IntoInnerError<W,>
  where Self: fmt::Debug, {
  #[inline]
  fn fmt(&self, fmt: &mut fmt::Formatter,) -> fmt::Result {
    fmt::Debug::fmt(self, fmt,)
  }
}

impl<W,> std::error::Error for IntoInnerError<W,>
  where Self: fmt::Debug + fmt::Display, {
  #[inline]
  fn source(&self,) -> Option<&(dyn 'static + std::error::Error)> { Some(&self.error) }
}

#[cfg(test,)]
mod tests {
  use super::*;

  #[test]
  fn test_bit_write() {
    let bytes = &mut [0, 0, 0xAD,][..];
    let mut bits = BitWrite::new(&mut bytes[..],);

    assert_eq!(bits.aligned(), true,);
    assert_eq!(bits.to_write(), 0,);
    assert!(bits.write_bit(true,).is_ok(),);
    assert!(bits.write_bit(true,).is_ok(),);
    assert!(bits.write_bit(false,).is_ok(),);
    assert!(bits.write_bits(0b1011_0110, 3,).is_ok(),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_write(), 2,);
    assert!(bits.write_bits(0, 2,).is_ok(),);
    assert_eq!(bits.aligned(), true,);
    assert_eq!(bits.to_write(), 0,);
    assert!(bits.write_bits(0b1011, 4,).is_ok(),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_write(), 4,);
    assert!(bits.write_bits(0b01, 2,).is_ok(),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_write(), 2,);
    assert!(bits.into_inner().is_ok(),);
    assert_eq!(bytes, &[0b1101_1000, 0b1011_0100, 0xAD,][..],);

    let mut bits = BitWrite::new([0u8; 0].as_mut(),);
    assert_eq!(bits.write_bits(0, 0,).ok(), Some(()),);
  }
}
