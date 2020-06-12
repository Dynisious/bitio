//! Author --- DMorgan  
//! Last Moddified --- 2019-12-31

use crate::{bits::Bits, UnalignedError,};
use core::{
  fmt,
  convert::TryFrom,
  borrow::BorrowMut,
};

mod tests;
#[cfg(feature = "std",)]
mod bit_writer;

#[cfg(feature = "std",)]
pub use self::bit_writer::*;
use alloc::vec::Vec;

/// A trait for bitwise writing.
pub trait BitWrite {
  /// The error type when writing bits, should include the number of bits left unwritten.
  type Error;

  /// Returns `true` if this writer is aligned to a byte.
  fn is_aligned(&self,) -> bool;
  /// Writes a single bit to the input.
  fn write_bit(&mut self, bit: bool,) -> Result<&mut Self, Self::Error> {
    self.write_bits(Bits::B1, bit as u8,).and(Ok(self),)
  }
  /// Writes a full byte to the input.
  /// 
  /// If an error is returned it could mean that there is not enough space for a full
  /// byte.
  #[inline]
  fn write_byte(&mut self, byte: u8,) -> Result<&mut Self, Self::Error> {
    self.write_bits(Bits::B8, byte,)?; Ok(self)
  }
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
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error>;
}

impl<W,> BitWrite for &'_ mut W
  where W: BitWrite, {
  type Error = W::Error;

  #[inline]
  fn is_aligned(&self,) -> bool { W::is_aligned(*self,) }
  #[inline]
  fn write_bit(&mut self, bit: bool,) -> Result<&mut Self, Self::Error> { W::write_bit(self, bit,)?; Ok(self) }
  fn write_byte(&mut self, byte: u8,) -> Result<&mut Self, Self::Error> { W::write_byte(self, byte,)?; Ok(self) }
  #[inline]
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error> { W::write_bits(*self, bits, buf,) }
}

/// Progressively fill a byte from high bits to low bits.
#[derive(Clone, Copy, Debug, Default,)]
pub struct WriteByte<B = u8,>
  where B: BorrowMut<u8>, {
  /// The store of bits being written.
  buffer: B,
  /// The cursor over the next bit to be written.
  cursor: Option<Bits>,
}

impl WriteByte<u8,> {
  /// An empty writer.
  pub const EMPTY: Self = Self::new(0,);

  /// The internal buffer is returned and the writer reset.
  pub fn reset(&mut self,) -> u8 {
    self.cursor = Some(Bits::B8);

    core::mem::replace(&mut self.buffer, 0,)
  }
}

impl<B,> WriteByte<B,>
  where B: BorrowMut<u8>, {
  /// Creates a new empty writer.
  pub const fn new(buffer: B,) -> Self { Self { buffer, cursor: Some(Bits::B8), } }
  /// Returns the number of bits left to write.
  #[inline]
  pub const fn to_write(&self,) -> Option<Bits> { self.cursor }
  /// Pads the internal buffer with zeros so that the writer is aligned.
  pub fn pad_zeros(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_write() {
      self.write_bits(bits, 0,).ok();
    }

    self
  }
  /// Unwraps the inner buffer if the writer is aligned.
  pub fn into_buffer(self,) -> Result<B, UnalignedError<Self,>> {
    match self.cursor {
      None | Some(Bits::B8) => Ok(self.buffer),
      Some(misalign) => Err(UnalignedError(self, misalign,)),
    }
  }
}

impl<B,> BitWrite for WriteByte<B,>
  where B: BorrowMut<u8>, {
  type Error = Bits;

  #[inline]
  fn is_aligned(&self,) -> bool { self.cursor.unwrap_or(Bits::B8,) == Bits::B8 }
  fn write_bit(&mut self, bit: bool,) -> Result<&mut Self, Self::Error> {
    //Get the cursor.
    let cursor = match self.cursor {
      Some(v) => v,
      None => return Err(Bits::B1),
    };

    //Set the bit.
    if bit { *self.buffer.borrow_mut() ^= cursor.bit() }
    //Advance the cursor.
    self.cursor = TryFrom::try_from((cursor as u8).wrapping_sub(1,),).ok();

    Ok(self)
  }
  fn write_bits(&mut self, bits: Bits, mut buf: u8,) -> Result<Bits, Self::Error> {
    //Get the number of bits the buffer is expecting to write.
    let to_write = match self.cursor {
      Some(v) => v,
      None => return Err(bits),
    };

    //Zero the high bits of the byte.
    buf &= bits.mask();
    //Write out the bits.
    if to_write >= bits {
      //There is enough space for all of the bits.

      //Calculate the shift to align the bits.
      let shift = to_write as u8 - bits as u8;

      //Write the bytes.
      *self.buffer.borrow_mut() |= buf << shift;
      //Update the cursor.
      self.cursor = Bits::try_from(shift,).ok();

      Ok(bits)
    } else {
      //There is not enough space for all of the bits.

      //Calculate the number of bits to shift out.
      let shift = unsafe { Bits::from_u8(bits as u8 - to_write as u8,) };
      //Write the bytes.
      *self.buffer.borrow_mut() |= buf >> shift as u8;
      //Update the cursor.
      self.cursor = None;

      Err(shift)
    }
  }
}

/// Progressively fill a slice from high bits to low bits.
/// 
/// The slices pointer will be updated as bytes are written.
#[derive(Debug,)]
pub struct WriteSlice<'s, B = u8,>
  where B: BorrowMut<u8>, {
  /// The store of bits being written.
  slice: &'s mut [B],
  /// The cursor over the next bit to be written.
  cursor: Bits,
}

impl<'s, B,> WriteSlice<'s, B,>
  where B: BorrowMut<u8>, {
  /// Creates a new empty writer.
  /// 
  /// # Params
  /// 
  /// slice --- The slice to fill.  
  pub const fn new(slice: &'s mut [B],) -> Self { Self { cursor: Bits::B8, slice, } }
  /// Returns the number of bits left to write before the writer is byte aligned.
  pub fn to_write(&self,) -> Option<Bits> {
    if self.cursor == Bits::B8 { None }
    else { Some(self.cursor) }
  }
  /// Pads the internal buffer with zeros so that the writer is aligned.
  pub fn pad_zeros(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_write() {
      self.write_bits(bits, 0,).ok();
    }

    self
  }
  /// Unwraps the unfilled portion of the inner slice if the writer is aligned.
  /// 
  /// If the slice is completly filled `None` is returned.
  pub fn into_slice(self,) -> Result<Option<&'s mut [B]>, UnalignedError<Self,>> {
    match self.cursor {
      Bits::B8 => Ok(
        if self.slice.is_empty() { None }
        else { Some(self.slice) }
      ),
      misalign => Err(UnalignedError(self, misalign,)),
    }
  }
}

impl<B,> BitWrite for WriteSlice<'_, B,>
  where B: BorrowMut<u8>, {
  type Error = Bits;

  #[inline]
  fn is_aligned(&self,) -> bool { self.cursor == Bits::B8 }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error> {
    //Get the byte being written too.
    let (buffer, slice,) = match unsafe { &mut *(self.slice as *mut [B]) }.split_first_mut() {
      Some(v) => v,
      None => return Err(bits),
    };
    //Create the byte buffer to write too.
    let mut buffer = WriteByte { buffer: buffer.borrow_mut(), cursor: Some(self.cursor), };

    //Write the bits out.
    match buffer.write_bits(bits, buf,) {
      //Update the cursor.
      Ok(bits) => {
        self.cursor = match buffer.cursor {
          Some(cursor) => cursor,
          //This byte is filled, advance the slice.
          None => { self.slice = slice; Bits::B8 },
        };

        Ok(bits)
      },
      //There are more bits to write.
      Err(to_write) => {
        self.cursor = Bits::B8;
        self.slice = slice;

        //Write the remaining bits.
        self.write_bits(to_write, buf,)
      },
    }
  }
}

/// Progressively fill a slice from high bits to low bits.
/// 
/// The slices pointer will be updated as bytes are written.
pub struct WriteIter<I,>
  where I: Iterator,
    I::Item: BorrowMut<u8>, {
  /// The iterator of bytes to write too.
  iter: I,
  /// The buffer of the byte being written too.
  buffer: Option<(Bits, I::Item,)>,
}

impl<I,> WriteIter<I,>
  where I: Iterator,
    I::Item: BorrowMut<u8>, {
  /// Creates a new empty writer.
  /// 
  /// # Params
  /// 
  /// slice --- The slice to fill.  
  pub fn new<Iter,>(iter: Iter,) -> Self
    where Iter: IntoIterator<IntoIter = I, Item = I::Item>, {
    Self { buffer: None, iter: iter.into_iter(), }
  }
  /// Returns the number of bits left to write before the writer is byte aligned.
  pub fn to_write(&self,) -> Option<Bits> { self.buffer.as_ref().map(|b,| b.0,) }
  /// Pads the internal buffer with zeros so that the writer is aligned.
  pub fn pad_zeros(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_write() {
      self.write_bits(bits, 0,).ok();
    }

    self
  }
  /// Unwraps the unfilled portion of the inner iterator if the writer is aligned.
  pub fn into_iter(self,) -> Result<I, UnalignedError<Self,>> {
    match self.to_write() {
      None => Ok(self.iter),
      Some(misalign) => Err(UnalignedError(self, misalign,)),
    }
  }
}

impl<I,> BitWrite for WriteIter<I,>
  where I: Iterator,
    I::Item: BorrowMut<u8>, {  
  type Error = Bits;

  #[inline]
  fn is_aligned(&self,) -> bool { self.buffer.is_none() }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error> {
    //Get the cursor and the byte to populate.
    let (cursor, mut byte,) = match core::mem::replace(&mut self.buffer, None,) {
      Some(v) => v,
      //Get the next byte to populate.
      None => self.iter.next().map(|b,| (Bits::B8, b,),).ok_or(bits,)?,
    };
    //Create the writer to write too.
    let mut buffer = WriteByte { cursor: Some(cursor), buffer: byte.borrow_mut(), };

    //Write the bits out.
    match buffer.write_bits(bits, buf,) {
      //Update the cursor.
      Ok(bits) => { self.buffer = buffer.cursor.map(move |c,| (c, byte,),); Ok(bits) },
      //Write the remaining bits.
      Err(to_write) => self.write_bits(to_write, buf,),
    }
  }
}

impl<I,> fmt::Debug for WriteIter<I,>
  where I: Iterator + fmt::Debug,
    I::Item: BorrowMut<u8> + fmt::Debug, {
  fn fmt(&self, fmt: &mut fmt::Formatter,) -> fmt::Result {
    fmt.debug_struct("WriteIter",)
    .field("iter", &self.iter,)
    .field("buffer", &self.buffer,)
    .finish()
  }
}

/// Progressively fill a `Vec` from high bits to low bits.
/// 
/// The slices pointer will be updated as bytes are written.
#[derive(Clone, Debug,)]
pub struct WriteVec {
  /// The store of bits being written.
  vec: Vec<u8>,
  /// The cursor over the next bit to be written.
  cursor: Bits,
}

impl WriteVec {
  /// Creates a new empty writer.  
  #[inline]
  pub const fn new() -> Self { Self { cursor: Bits::B8, vec: Vec::new(), } }
  /// Returns the number of bits left to write before the writer is byte aligned.
  pub fn to_write(&self,) -> Option<Bits> { Some(self.cursor).filter(|&b,| b != Bits::B8,) }
  /// Pads the internal buffer with zeros so that the writer is aligned.
  pub fn pad_zeros(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_write() {
      self.write_bits(bits, 0,).ok();
    }

    self
  }
  /// Unwraps the inner `Vec` if the writer is aligned.
  pub fn into_vec(self,) -> Result<Vec<u8>, UnalignedError<Self,>> {
    match self.cursor {
      Bits::B8 => Ok(self.vec),
      misalign => Err(UnalignedError(self, misalign,))
    }
  }
}

impl BitWrite for WriteVec {
  type Error = !;

  #[inline]
  fn is_aligned(&self,) -> bool { self.cursor == Bits::B8 }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error> {
    //Get the byte to write too.
    let buffer = if self.cursor == Bits::B8 {
      //We are starting a new byte.

      let last = self.vec.len();
      self.vec.push(0,);

      &mut self.vec[last]
    } else {
      //We are continuing an old byte.

      let last = self.vec.len() - 1;

      &mut self.vec[last]
    };
    //Get the writer to write too.
    let mut buffer = WriteByte { cursor: Some(self.cursor), buffer, };

    //Write the bits out.
    match buffer.write_bits(bits, buf,) {
      //Update the cursor.
      Ok(bits) => { self.cursor = buffer.cursor.unwrap_or(Bits::B8,); Ok(bits) },
      //Write the remaining bits.
      Err(to_write) => { self.cursor = Bits::B8; self.write_bits(to_write, buf,) },
    }
  }
}
