//! Author --- daniel.bechaz@gmail.com  
//! Last Moddified --- 2019-07-24

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
  /// Writes the internal 
  fn flush_buf(&mut self,) -> io::Result<()> {
    use std::{mem, slice,};

    self.inner.write(
      slice::from_ref(&mem::replace(&mut self.buffer, 0,),),
    )?;

    Ok(())
  }
  /// Advances the internal cursor and returns `true` if the cursor is now alligned.
  fn advance_cursor(&mut self,) -> io::Result<()> {
    //Advance the cursor.
    self.cursor >>= 1;

    //If the cursor is aligned, reset the cursor.
    let aligned = self.cursor == 0;
    if aligned {
      self.cursor = Self::CURSOR_INIT;

      //Flush the buffer.
      self.flush_buf()?;
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
  pub fn to_write(&self,) -> u32 {
    let zeros = self.cursor.trailing_zeros();

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
    self.advance_cursor()?;

    Ok(())
  }
  /// Writes the bits in `buffer` to the internal writer.
  /// 
  /// # Param
  /// 
  /// buffer --- The buffer of bits, stored in the lower bits.  
  /// bits --- The number of bits from `buffer` to write.  
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
    use std::mem;

    assert!(0 < bits && bits <= 8, "0 >= `bits` > 8",);

    let to_write = self.to_write() as u8;

    if bits < to_write {
      //The difference between `to_write` and `bits`.
      let shift = to_write - bits;

      //Advance the cursor.
      self.cursor >>= bits;

      //Shift the bits in the buffer to allign with the bits to be written.
      buffer <<= shift;

      //Add the bits to the buffer.
      self.buffer |= self.buffer ^ buffer;
    } else {
      //The difference between `to_write` and `bits`.
      let shift = bits - to_write;

      //Add the bits to the buffer.
      let buf = self.buffer | self.buffer ^ (buffer >> shift);
      let buf = mem::replace(&mut self.buffer, buf,);

      //Flush out the written bits.
      if let Err(e) = self.flush_buf() {
        //Reset the buffer.
        self.buffer = buf;

        return Err(e);
      }

      //Set the cursor as needed.
      self.cursor = Self::CURSOR_INIT >> shift;

      //Set the buffer.
      self.buffer = buffer.wrapping_shl(8 - shift as u32,);
    }

    Ok(())
  }
  /// Returns the inner writer flushing any pending bits in buffer right padded with zeros.
  pub fn into_inner(mut self,) -> Result<W, IntoInnerError<W>> {
    use std::slice;
    
    if !self.aligned() {
      //Flush the buffer.
      if let Err(error) = self.inner.write(slice::from_ref(&self.buffer,),) {
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
    let bytes = &mut [0, 0xAD,][..];
    let mut bits = BitWrite::new(&mut bytes[..],);

    assert_eq!(bits.aligned(), true,);
    assert!(bits.write_bit(true,).is_ok(),);
    assert!(bits.write_bit(true,).is_ok(),);
    assert!(bits.write_bit(false,).is_ok(),);
    assert!(bits.write_bits(0b110, 3,).is_ok(),);
    assert_eq!(bits.aligned(), false,);
    assert_eq!(bits.to_write(), 2,);
    assert!(bits.into_inner().is_ok(),);
    assert_eq!(bytes, &[0b1101_1000, 0xAD,][..],);
  }
}
