//! Author --- DMorgan  
//! Last Moddified --- 2019-12-29

use super::*;
use std::io::{self, Read, Write, ErrorKind, Error,};

/// Wraps an IO reader and reads from it bitwise, high bits first.
#[derive(Clone, Copy, Debug, Hash,)]
pub struct ReadIO<R,>
  where R: Read, {
  /// The current byte being read.
  buffer: ReadByte,
  /// The reader of bytes to read.
  reader: R,
}

impl<R,> ReadIO<R,>
  where R: Read, {
  /// Constructs a new `ReadIO` over the reader.
  /// 
  /// # Params
  /// 
  /// reader --- The reader to read from.  
  #[inline]
  pub const fn new(reader: R,) -> Self {
    Self { reader, buffer: ReadByte::EMPTY, }
  }
  /// Returns the number of bytes left to read before this reader is aligned.
  #[inline]
  pub fn to_read(&self,) -> Option<Bits> { self.buffer.to_read() }
  /// Skips some bits cheaply.
  /// 
  /// There is no issue with skipping more bits than are in the buffer.
  /// 
  /// # Params
  /// 
  /// bits --- The number of bits to skip.  
  pub fn skip(&mut self, bits: Bits,) -> Result<&mut Self, Error> {
    //The number of bits currently in the buffer.
    let available = Bits::as_u8(self.buffer.to_read(),);

    //Skip the bits in the current buffer.
    self.buffer.skip(bits,).ok();
    //If there were enough bits in the buffer stop.
    if bits <= available { return Ok(self) }

    //There were not enough bits in the buffer, get the next bit and continue.

    //Read in the next byte.
    let next_byte = {
      let mut next_byte = 0;
      let next_byte_buf = core::slice::from_mut(&mut next_byte,);

      match self.reader.read_exact(next_byte_buf,) {
        Ok(_) => Some(next_byte),
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => None,
        Err(e) => return Err(e),
      }
    };
    //Repopulate the buffer.
    match next_byte {
      //Populate the buffer.
      Some(v) => { self.buffer.set(v,); },
      //There is no more data, stop.
      None => return Ok(self),
    }

    //Skip the unskipped bits.
    self.buffer.skip(unsafe { Bits::from_u8(bits as u8 - available,) },).ok();

    Ok(self)
  }
  /// Unwraps the inner reader if the inner buffer is empty.
  pub fn into_reader(self,) -> Result<R, UnalignedError<Self,>> {
    match self.buffer.cursor {
      None => Ok(self.reader),
      Some(misalign) => Err(UnalignedError(self, misalign,))
    }
  }
}

impl<R,> BitRead for ReadIO<R,>
  where R: Read, {
  /// The number of bytes available to read and the error encountered.
  type Error = (Option<Bits>, Error,);

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
    let next_byte = {
      //A buffer for the next byte.
      let mut next_byte = 0;
      let next_byte_buf = core::slice::from_mut(&mut next_byte,);

      //Read the byte.
      self.reader.read_exact(next_byte_buf,)
      .map_err(move |e,| (available, e,),)?;

      next_byte
    };
    //The number of bits which need to be read from the next byte.
    let remaining = unsafe { Bits::from_u8(bits as u8 - Bits::as_u8(available,),) };
    //Get the high bits from the current buffer and shift them into the higher bits of
    //the output.
    let high_bits = self.buffer.buffer << remaining as u8;
    //Get the low bits from the next byte.
    let low_bits = {
      //Populate the buffer with the next byte and skip the bits being read now.
      self.buffer.set(next_byte,).skip(remaining,).ok();

      //Read the bits and shift them into the lower bits of the output.
      //Apply the mask to clear the high bits of the part.
      (self.buffer.buffer >> (8 - remaining as u8)) & remaining.mask()
    };

    //Combine the bits in the output.
    Ok(high_bits ^ low_bits)
  }
}

impl<R,> Write for ReadIO<R,>
  where R: Read + Write, {
  #[inline]
  fn flush(&mut self,) -> io::Result<()> { self.reader.flush() }
  #[inline]
  fn write(&mut self, buffer: &[u8],) -> io::Result<usize> { self.reader.write(buffer,) }
}
