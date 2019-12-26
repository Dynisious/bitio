//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

use crate::{UnalignedError, bits::Bits,};
use core::convert::TryFrom;

mod tests;
#[cfg(feature = "std",)]
mod bit_reader;

#[cfg(feature = "std",)]
pub use self::bit_reader::*;

/// A trait for bitwise reading.
pub trait BitRead {
  /// The error type when reading bits.
  type Error;

  /// Returns `true` if this reader is aligned to a byte.
  fn is_aligned(&self,) -> bool;
  /// Reads a single bit from the input.
  fn read_bit(&mut self,) -> Result<bool, Self::Error> {
    self.read_bits(Bits::B1,).map(#[inline] |b,| b == 1,)
  }
  /// Reads a full byte from the input.
  /// 
  /// If an error is returned it could mean that less than a full byte is available.
  #[inline]
  fn read_byte(&mut self,) -> Result<u8, Self::Error> { self.read_bits(Bits::B8,) }
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
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash,)]
pub struct ReadByte {
  /// The bits being read from.
  buffer: u8,
  /// The cursor over the next bit to be read.
  cursor: Option<Bits>,
}

impl ReadByte {
  /// An empty reader.
  pub const EMPTY: Self = Self { buffer: 0, cursor: None, };

  /// Reads the bits from `buffer`.
  /// 
  /// # Params
  /// 
  /// buffer --- The byte to read bits from.  
  #[inline]
  pub const fn new(buffer: u8,) -> Self {
    Self { cursor: Some(Bits::B8), buffer, }
  }
  /// Returns the number of bits left to read.
  #[inline]
  pub fn to_read(&self,) -> u8 { Bits::as_u8(self.cursor,) }
  /// Resets the reader and fills the buffer.
  /// 
  /// # Params
  /// 
  /// buffer --- The new bits for the buffer.  
  pub fn set(&mut self, buffer: u8,) -> &mut Self {
    self.buffer = buffer;
    self.cursor = Some(Bits::B8);

    self
  }
  /// Skips some bits cheaply.
  /// 
  /// There is no issue with skipping more bits than are in the buffer.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to skip.  
  pub fn skip(&mut self, bits: Bits,) -> &mut Self {
    self.cursor = Bits::try_from(Bits::as_u8(self.cursor,).wrapping_sub(bits as u8,),).ok();

    self
  }
  /// Unwraps the inner buffer if the reader is aligned.
  /// 
  /// If the buffer has been completly consumed `None` is returned.
  pub fn into_buffer(self,) -> Result<Option<u8>, UnalignedError<Self,>> {
    //If the reader is aligned, return the buffer.
    if self.is_aligned() { Ok(self.cursor.and(Some(self.buffer),)) }
    else { Err(UnalignedError(self,)) }
  }
}

impl BitRead for ReadByte {
  type Error = u8;

  #[inline]
  fn is_aligned(&self,) -> bool {
    self.cursor == Some(Bits::B8) || self.cursor == None
  }
  fn read_bit(&mut self,) -> Result<bool, Self::Error> {
    //Read the bit.
    let res = self.cursor.ok_or(0,)?.bit() & self.buffer != 0;

    //Advance the cursor.
    self.cursor = unsafe { core::mem::transmute(Bits::as_u8(self.cursor,) - 1,) };

    Ok(res)
  }
  fn read_byte(&mut self,) -> Result<u8, Self::Error> {
    //If we are reading an entire byte the cursor must be fresh.
    if self.cursor == Some(Bits::B8) {
      self.cursor = None;

      Ok(self.buffer)
    //There are not enough bits, return the number of bits available.
    } else { Err(self.to_read()) }
  }
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error> {
    //Get the cursor.
    let cursor = self.cursor.ok_or(0,)?;
    //The shift applied to the buffer to populate the low bits.
    let shift = {
      //If there are not enough bits available error.
      if cursor < bits { return Err(cursor as u8) }

      cursor - bits as u8
    };

    //Advance the cursor.
    self.cursor = Bits::try_from(shift,).ok();

    Ok(self.buffer >> shift as u8)
  }
}

impl From<u8> for ReadByte {
  #[inline]
  fn from(from: u8,) -> Self { Self::new(from,) }
}

/// Wraps an iterator of bytes and reads from it bitwise, high bits first.
#[derive(Clone, Copy, Debug, Hash,)]
pub struct ReadIter<I,>
  where I: Iterator<Item = u8>, {
  /// The iterator of bytes to read.
  iterator: I,
  /// The current byte being read.
  buffer: ReadByte,
}

impl<I,> ReadIter<I,>
  where I: Iterator<Item = u8>, {
  /// Constructs a new `ReadIter` over the iterator.
  /// 
  /// # Params
  /// 
  /// iter --- The iterator to read from.  
  #[inline]
  pub fn new<Iter,>(iter: Iter,) -> Self
    where Iter: IntoIterator<IntoIter = I, Item = u8>, {
    Self { iterator: iter.into_iter(), buffer: ReadByte::EMPTY, }
  }
  /// Returns the number of bytes left to read before this reader is aligned.
  #[inline]
  pub fn to_read(&self,) -> u8 { self.buffer.to_read() }
  /// Skips some bits cheaply.
  /// 
  /// There is no issue with skipping more bits than are in the buffer.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to skip.  
  pub fn skip(&mut self, bits: Bits,) -> &mut Self {
    //The number of bits currently in the buffer.
    let available = self.buffer.to_read();

    //Skip the bits in the current buffer.
    self.buffer.skip(bits,);
    //If there were enough bits in the buffer stop.
    if bits <= available { return self }

    //There were not enough bits in the buffer, get the next bit and continue.

    //Repopulate the buffer.
    match self.iterator.next() {
      //Populate the buffer.
      Some(v) => { self.buffer.set(v,); },
      //There is no more data, stop.
      None => return self,
    }

    //Skip the unskipped bits.
    self.buffer.skip(unsafe { Bits::from_u8(bits - available,) },);

    self
  }
  /// Unwraps the inner iterator if the reader is aligned.
  pub fn into_iter(self,) -> Result<I, UnalignedError<Self,>> {
    if self.buffer.is_aligned() { Ok(self.iterator) }
    else { Err(UnalignedError(self,)) }
  }
}

impl<I,> BitRead for ReadIter<I,>
  where I: Iterator<Item = u8>, {
  type Error = u8;

  #[inline]
  fn is_aligned(&self,) -> bool { self.buffer.is_aligned() }
  fn read_bits(&mut self, bits: Bits,) -> Result<u8, Self::Error> {
    //Attempt to read the bits from the buffer, store the number of bits in the buffer if
    //not enough are avaialable.
    let available = match self.buffer.read_bits(bits,) {
      //Return the bits read.
      Ok(v) => return Ok(v),
      Err(v) => v,
    };
    //Read in the next byte.
    let next_byte = self.iterator.next().ok_or(available,)?;
    //The number of bits which need to be read from the next byte.
    let remaining = unsafe { Bits::from_u8(bits - available,) };
    //Get the high bits from the current buffer and shift them into the higher bits of
    //the output.
    let high_bits = self.buffer.buffer << remaining as u8;
    //Get the low bits from the next byte.
    let low_bits = {
      //Populate the buffer with the next byte and skip the bits being read now.
      self.buffer.set(next_byte,).skip(remaining,);

      //Read the bits and shift them into the lower bits of the output.
      //Apply the mask to clear the high bits of the part.
      (self.buffer.buffer >> (Bits::B8 - remaining) as u8) & remaining.mask()
    };

    //Combine the bits in the output.
    Ok(high_bits ^ low_bits)
  }
}
