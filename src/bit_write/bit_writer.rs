//! Author --- DMorgan  
//! Last Moddified --- 2019-12-26

use super::*;
use std::io::{self, Read, Write, Error,};

/// Wraps an IO writer and writes to it bitwise, high bits first.
#[derive(Clone, Copy, Debug, Hash,)]
pub struct WriteIO<W,>
  where W: Write, {
  /// The current byte being read.
  buffer: WriteByte,
  /// The writer of bytes to write too.
  writer: W,
}

impl<W,> WriteIO<W,>
  where W: Write, {
  /// Constructs a new `WriteIO` over the writer.
  /// 
  /// # Params
  /// 
  /// writer --- The writer to write too.  
  #[inline]
  pub const fn new(writer: W,) -> Self {
    Self { writer, buffer: WriteByte::EMPTY, }
  }
  /// The number of bytes before the writer is byte aligned.
  #[inline]
  pub fn to_write(&self,) -> u8 { self.buffer.to_write() }
  /// Attempts to clear the internal buffer if it is full.
  pub fn flush(&mut self,) -> io::Result<()> {
    match self.buffer.reset() {
      Some(byte) => self.writer.write_all(core::slice::from_ref(&byte,),),
      None => Ok(()),
    }
  }
  /// Clears the internal buffer, returning its current state.
  #[inline]
  pub fn clear_buffer(&mut self,) -> WriteByte {
    core::mem::replace(&mut self.buffer, WriteByte::EMPTY,)
  }
  /// Unwraps the inner writer if the inner buffer is empty and the writter is flushed.
  /// 
  /// If an unaligned error is returned either write the number of bytes returned by
  /// `to_write` or try `flush`ing or `clear`ing the writer. 
  pub fn into_writer(self,) -> Result<W, UnalignedError<Self,>> {
    if self.buffer.cursor == Some(Bits::B8) { Ok(self.writer) }
    else { Err(UnalignedError(self,)) }
  }
}

impl<W,> BitWrite for WriteIO<W,>
  where W: Write, {
  type Error = (u8, Error,);

  #[inline]
  fn is_aligned(&self,) -> bool { self.buffer.cursor == Some(Bits::B8) }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<u8, Self::Error> {
    //The number of bits written to the internal buffer.
    let written = self.buffer.write_bits(bits, buf,)?;

    if self.buffer.to_write() == 0 {
      //The internal buffer has been filled.

      //Write out the byte.
      self.writer.write_all(core::slice::from_ref(&self.buffer.buffer,),)
        .map_err(move |e,| (written, e,),)?;
      //Reset the internal buffer.
      self.buffer = WriteByte::EMPTY;
    }

    //Calculate the bits which are yet to be written.
    if let Ok(bits) = Bits::try_from(bits - written,) {
      //There are unwritten bits.

      //Write out the remaining bits.
      self.buffer.write_bits(bits, buf,)?;
    }

    Ok(bits as u8)
  }
}

impl<W,> Read for WriteIO<W,>
  where W: Read + Write, {
  #[inline]
  fn read(&mut self, buf: &mut [u8],) -> io::Result<usize> { W::read(&mut self.writer, buf,) }
}