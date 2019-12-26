//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

use crate::{bits::Bits, UnalignedError,};
use core::convert::TryFrom;

mod tests;
#[cfg(feature = "std",)]
mod bit_writer;

#[cfg(feature = "std",)]
pub use self::bit_writer::*;

/// A trait for bitwise writing.
pub trait BitWrite {
  /// The error type when writing bits.
  type Error;

  /// Returns `true` if this writer is aligned to a byte.
  fn is_aligned(&self,) -> bool;
  /// Writes a single bit to the input.
  fn write_bit(&mut self, bit: bool,) -> Result<bool, Self::Error> {
    self.write_bits(Bits::B1, bit as u8,).map(|b,| b != 0,)
  }
  /// Writes a full byte to the input.
  /// 
  /// If an error is returned it could mean that there is not enough space for a full
  /// byte.
  #[inline]
  fn write_byte(&mut self, byte: u8,) -> Result<u8, Self::Error> { self.write_bits(Bits::B8, byte,) }
  /// Writes low bits from `buf` to the input in bulk.
  /// 
  /// The state of the higher bits are ignored.
  /// 
  /// Attempting to write too many bits should return how many bits were written.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to write out.  
  /// buf --- The buffer of bits to write.  
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<u8, Self::Error>;
}

impl<W,> BitWrite for &'_ mut W
  where W: BitWrite, {
  type Error = W::Error;

  #[inline]
  fn is_aligned(&self,) -> bool { W::is_aligned(*self,) }
  #[inline]
  fn write_bit(&mut self, bit: bool,) -> Result<bool, Self::Error> { W::write_bit(self, bit,) }
  #[inline]
  fn write_byte(&mut self, byte: u8,) -> Result<u8, Self::Error> { W::write_byte(self, byte,) }
  #[inline]
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<u8, Self::Error> { W::write_bits(*self, bits, buf,) }
}

/// Progressively fill a byte from high bits to low bits.
#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash,)]
pub struct WriteByte {
  /// The store of bits being written.
  buffer: u8,
  /// The cursor over the next bit to be written.
  cursor: Option<Bits>,
}

impl WriteByte {
  /// An empty writer.
  pub const EMPTY: Self = Self { buffer: 0, cursor: Some(Bits::B8), };

  /// Creates a new empty writer.
  #[inline]
  pub const fn new() -> Self { Self::EMPTY }
  /// Returns the number of bits left to write.
  #[inline]
  pub fn to_write(&self,) -> u8 { Bits::as_u8(self.cursor,) }
  /// If the internal buffer is full its content is returned and the writer reset.
  pub fn reset(&mut self,) -> Option<u8> {
    match self.cursor {
      Some(_) => None,
      None => {
        self.cursor = Some(Bits::B8);
        
        Some(core::mem::replace(&mut self.buffer, 0,))
      },
    }
  }
  /// Unwraps the inner buffer if the writer is aligned.
  /// 
  /// If the buffer is completly empty `None` is returned.
  pub fn into_buffer(self,) -> Result<Option<u8>, UnalignedError<Self,>> {
    if self.cursor == None { Ok(Some(self.buffer)) }
    else if self.cursor == Some(Bits::B8) { Ok(None) }
    else { Err(UnalignedError(self,)) }
  }
}

impl BitWrite for WriteByte {
  type Error = !;

  #[inline]
  fn is_aligned(&self,) -> bool {
    self.cursor == Some(Bits::B8) || self.cursor == None
  }
  fn write_bit(&mut self, bit: bool,) -> Result<bool, Self::Error> {
    //Get the cursor.
    let cursor = match self.cursor {
      Some(v) => v,
      None => return Ok(false),
    };

    //Set the bit.
    if bit { self.buffer ^= cursor.bit() }
    //Advance the cursor.
    self.cursor = TryFrom::try_from((cursor as u8).wrapping_sub(1,),).ok();

    Ok(true)
  }
  fn write_bits(&mut self, bits: Bits, mut buf: u8,) -> Result<u8, Self::Error> {
    //Get the number of bits the buffer is expecting to write.
    let to_write = match self.cursor {
      Some(v) => v,
      None => return Ok(0),
    };
    //The number of bits being written to the internal buffer.
    let writing = {
      if to_write <= bits {
        //There are enough bits to fill the buffer.

        //Calculate the shift to align the bits with the low bits of the buffer.
        let shift = bits - to_write as u8;
        //Shift the buffer to align the bits.
        buf >>= shift;

        to_write
      } else {
        //There aren't enough bits to fill the buffer.

        //Calculate the shift to align the bits with the destination bits of the buffer.
        let shift = to_write - bits as u8;
        //Shift the buffer to align the bits.
        buf <<= shift;

        bits
      }
    };

    //Clear the high bits of the buf.
    buf &= to_write.mask();
    //Add the bits to the internal buffer.
    self.buffer ^= buf;
    //Advance the cursor.
    self.cursor = Bits::try_from(to_write - writing as u8,).ok();

    Ok(writing as u8)
  }
}

/// Progressively fill a slice from high bits to low bits.
/// 
/// The slices pointer will be updated as bytes are written.
#[derive(PartialEq, Eq, Debug, Hash,)]
pub struct WriteSlice<'s,> {
  /// The store of bits being written.
  slice: &'s mut [u8],
  /// The cursor over the next bit to be written.
  cursor: Option<Bits>,
}

impl<'s,> WriteSlice<'s,> {
  /// Creates a new empty writer.
  /// 
  /// # Params
  /// 
  /// slice --- The slice to fill.  
  pub fn new(slice: &'s mut [u8],) -> Self {
    Self {
      cursor: Some(Bits::B8)
        //Clear the cursor if the slice is filled.
        .filter(|_,| slice.is_empty() == false,),
      slice,
    }
  }
  /// Returns the number of bits left to write.
  #[inline]
  pub fn to_write(&self,) -> u8 { Bits::as_u8(self.cursor,) }
  /// Unwraps the unfilled portion of the inner slice if the writer is aligned.
  /// 
  /// If the slice is completly filled `None` is returned.
  pub fn into_slice(self,) -> Result<Option<&'s mut [u8]>, UnalignedError<Self,>> {
    if self.cursor == Some(Bits::B8) { Ok(Some(self.slice)) }
    else if self.cursor == None { Ok(None) }
    else { Err(UnalignedError(self,)) }
  }
}

impl BitWrite for WriteSlice<'_,> {
  type Error = !;

  #[inline]
  fn is_aligned(&self,) -> bool {
    self.cursor == Some(Bits::B8) || self.cursor == None
  }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<u8, Self::Error> {
    //If the slice is empty stop.
    if self.slice.is_empty() { return Ok(0) }

    //The length of the slice.
    let len = self.slice.len();
    //The byte being written too.
    let mut byte = &mut self.slice[0];
    //Create a buffer for the current byte.
    let mut buffer = WriteByte { cursor: self.cursor, buffer: *byte, };
    //The number of bits written to the internal buffer.
    let written = buffer.write_bits(bits, buf,)?;

    //Write out the byte.
    *byte = buffer.buffer;
    if buffer.to_write() == 0 {
      //The internal buffer has been filled.

      //Advance the byte pointer.
      byte = unsafe { &mut *(byte as *mut u8).add(1,) };
      //Advance the inner slice.
      self.slice = unsafe { core::slice::from_raw_parts_mut(byte, len - 1,) };
      if self.slice.is_empty() {
        //The slice is full.

        //Clear the cursor.
        self.cursor = None;
        return Ok(written);
      }
    }

    //Calculate the bits which are yet to be written.
    if let Ok(bits) = Bits::try_from(bits - written,) {
      //There are unwritten bits.

      //The number of bits to be written after these bits.
      let to_write = Bits::B8 - bits;
      //Store the pending bits.
      *byte = buf << to_write as u8;
      //Update the cursor.
      self.cursor = Some(to_write);
    //Update the cursor.
    } else { self.cursor = buffer.cursor.or(Some(Bits::B8),) }

    Ok(bits as u8)
  }
}