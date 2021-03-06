//! Author --- DMorgan  
//! Last Moddified --- 2019-12-31

use crate::{UnalignedError, Bits,};
use core::{
  convert::TryFrom,
  borrow::Borrow,
};

mod tests;
#[cfg(feature = "std",)]
mod bit_reader;

#[cfg(feature = "std",)]
pub use self::bit_reader::*;

/// A trait for bitwise reading.
pub trait BitRead {
  /// The error type when reading bits, should include the number of bits available.
  type Error;

  /// Returns `true` if this reader is aligned to a byte.
  fn is_aligned(&self,) -> bool;
  /// Reads a single bit from the input.
  fn read_bit(&mut self,) -> Result<bool, Self::Error> {
    self.read_bits(Bits::B1,).map(move |b,| (b & Bits::B1.bit()) != 0,)
  }
  /// Reads a full byte from the input.
  /// 
  /// If an error is returned it could mean that less than a full byte is available.
  #[inline]
  fn read_byte(&mut self,) -> Result<u8, Self::Error> { self.read_bits(Bits::B8,) }
  /// Reads in bits without returning them.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to read in.  
  fn skip(&mut self, bits: Bits,) -> Result<&mut Self, Self::Error> {
    self.read_bits(bits,)?; Ok(self)
  }
  /// Reads several bits from the input at once.
  /// 
  /// The state of the higher bits are not enforced.
  /// 
  /// The bits will occupy the low bits of the returned byte.  
  /// `bits.mask()` will return a mask corresponding to the correct bits.  
  /// 
  /// Attempting to read too many bits should not remove any bits from the input and
  /// instead inform how many bits are available.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to read off.  
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error>;
}

impl<R,> BitRead for &'_ mut R
  where R: BitRead, {
  type Error = R::Error;

  #[inline]
  fn is_aligned(&self,) -> bool { R::is_aligned(*self,) }
  #[inline]
  fn read_bit(&mut self,) -> Result<bool, Self::Error> { R::read_bit(self,) }
  #[inline]
  fn read_byte(&mut self,) -> Result<u8, Self::Error> { R::read_byte(self,) }
  #[inline]
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error> { R::read_bits(*self, bits,) }
}

/// Wraps a byte and reads from it high bits first.
#[derive(Clone, Copy, Debug,)]
pub struct ReadByte<B = u8,>
  where B: Borrow<u8>, {
  /// The bits being read from.
  buffer: B,
  /// The cursor over the next bit to be read.
  cursor: Option<Bits>,
}

impl ReadByte<u8,> {
  /// An empty reader.
  pub const EMPTY: Self = Self { buffer: 0, cursor: None, };
}

impl<B,> ReadByte<B,>
  where B: Borrow<u8>, {
  /// Reads the bits from `buffer`.
  /// 
  /// # Params
  /// 
  /// buffer --- The byte to read bits from.  
  pub const fn new(buffer: B,) -> Self {
    Self { cursor: Some(Bits::B8), buffer, }
  }
  /// Returns the number of bits left to read.
  #[inline]
  pub const fn to_read(&self,) -> Option<Bits> { self.cursor }
  /// Resets the reader and sets the buffer.
  /// 
  /// # Params
  /// 
  /// buffer --- The new bits for the buffer.  
  pub fn set(&mut self, buffer: B,) -> &mut Self {
    self.buffer = buffer;
    self.cursor = Some(Bits::B8);

    self
  }
  /// Clears the internal byte buffer so that this reader is aligned.
  pub fn clear_buf(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_read() {
      self.skip(bits,).ok();
    }

    self
  }
  /// Unwraps the inner buffer if the reader is aligned.
  pub fn into_buffer(self,) -> Result<B, UnalignedError<Self,>> {
    match self.cursor {
      None | Some(Bits::B8) => Ok(self.buffer),
      Some(misalign) => Err(UnalignedError(self, misalign,)),
    }
  }
}

impl<B,> BitRead for ReadByte<B,>
  where B: Borrow<u8>, {
  type Error = Option<Bits>;

  fn is_aligned(&self,) -> bool {
    self.cursor.unwrap_or(Bits::B8,) == Some(Bits::B8)
  }
  fn read_bit(&mut self,) -> Result<bool, Self::Error> {
    //Get the bit being read.
    let cursor = self.cursor.ok_or(None,)?;
    //Read the bit.
    let res = cursor.bit() & *self.buffer.borrow() != 0u8;

    //Advance the cursor.
    self.cursor = Bits::try_from(cursor as u8 - 1,).ok();

    Ok(res)
  }
  fn read_byte(&mut self,) -> Result<u8, Self::Error> {
    match self.cursor {
      //If we are reading an entire byte the cursor must be fresh.
      Some(Bits::B8) => {
        self.cursor = None;

        Ok(*self.buffer.borrow())
      },
      //There are not enough bits, return the number of bits available.
      remaining => Err(remaining),
    }
  }
  fn skip(&mut self, bits: Bits,) -> Result<&mut Self, Self::Error> {
    self.cursor = Bits::try_from(Bits::as_u8(self.cursor,).wrapping_sub(bits as u8,),).ok();

    Ok(self)
  }
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error> {
    //Get the cursor.
    let cursor = self.cursor.filter(|&b,| b >= bits,).ok_or(self.cursor,)?;
    //The shift applied to the buffer to populate the low bits.
    let shift = cursor as u8 - bits as u8;

    //Advance the cursor.
    self.cursor = Bits::try_from(shift,).ok();

    Ok(self.buffer.borrow().wrapping_shr(shift as u32,))
  }
}

impl<B,> From<B> for ReadByte<B,>
  where B: Borrow<u8>, {
  #[inline]
  fn from(from: B,) -> Self { Self::new(from,) }
}

/// Wraps an iterator of bytes and reads from it bitwise, high bits first.
#[derive(Clone, Copy, Debug,)]
pub struct ReadIter<I,>
  where I: Iterator,
    I::Item: Borrow<u8>, {
  /// The iterator of bytes to read.
  iterator: I,
  /// The current byte being read.
  buffer: ReadByte<u8>,
}

impl<I,> ReadIter<I,>
  where I: Iterator,
    I::Item: Borrow<u8>, {
  /// Constructs a new `ReadIter` over the iterator.
  /// 
  /// # Params
  /// 
  /// iter --- The iterator to read from.  
  pub fn new<Iter,>(iter: Iter,) -> Self
    where Iter: IntoIterator<IntoIter = I, Item = I::Item>, {
    Self { iterator: iter.into_iter(), buffer: ReadByte::EMPTY, }
  }
  /// Returns the number of bytes left to read before this reader is aligned.
  #[inline]
  pub fn to_read(&self,) -> Option<Bits> { self.buffer.to_read() }
  /// Clears the internal byte buffer so that this reader is aligned.
  pub fn clear_buf(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_read() {
      self.skip(bits,).ok();
    }

    self
  }
  /// Unwraps the inner iterator if the reader is aligned.
  pub fn into_iter(self,) -> Result<I, UnalignedError<Self,>> {
    match self.buffer.to_read() {
      Some(misalign) => Err(UnalignedError(self, misalign,)),
      None => Ok(self.iterator),
    }
  }
}

impl<I,> BitRead for ReadIter<I,>
  where I: Iterator,
    I::Item: Borrow<u8>, {
  type Error = Option<Bits>;

  #[inline]
  fn is_aligned(&self,) -> bool { self.buffer.is_aligned() }
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error> {
    //Attempt to read the bits from the buffer.
    let available = match self.buffer.read_bits(bits,) {
      //Return the bits read.
      Ok(v) => return Ok(v),
      Err(v) => v,
    };
    //Read in the next byte.
    let next_byte = *self.iterator.next().ok_or(available,)?.borrow();

    //Read the bits.
    if let Some(available) = available {
      //The number of bits which need to be read from the next byte.
      let remaining = unsafe { Bits::from_u8(bits as u8 - available as u8,) };
      //Get the high bits from the current buffer and shift them into the higher bits of
      //the output.
      let high_bits = self.buffer.buffer << remaining as u8;
      //Get the low bits from the next byte.
      let low_bits = {
        //Populate the buffer with the next byte and skip the bits being read now.
        self.buffer.set(next_byte,).skip(remaining,).ok();

        //Read the bits and shift them into the lower bits of the output.
        //Apply the mask to clear the high bits of the part.
        self.buffer.buffer >> (8 - remaining as u8)
      };

      //Combine the bits in the output.
      Ok(high_bits | low_bits)
    //Reset the byte and continue reading.
    } else { self.buffer.set(next_byte,).read_bits(bits,) }
  }
}
