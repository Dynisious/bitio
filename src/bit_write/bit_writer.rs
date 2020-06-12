//! Author --- DMorgan  
//! Last Moddified --- 2019-12-31

use super::*;
use std::io::{self, Read, Write, Error,};

/// Wraps an IO writer and writes to it bitwise, high bits first.
#[derive(Clone, Copy, Debug,)]
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
  pub fn to_write(&self,) -> Option<Bits> { self.buffer.to_write().filter(|&b,| b != Bits::B8,) }
  /// Pads the internal buffer with zeros so that the writer is aligned.
  pub fn pad_zeros(&mut self,) -> &mut Self {
    if let Some(bits) = self.to_write() {
      self.write_bits(bits, 0,).ok();
    }

    self
  }
  /// Attempts to clear the internal buffer if it is full.
  pub fn flush(&mut self,) -> io::Result<&mut Self> {
    if self.buffer.cursor == None {
      self.writer.write_all(core::slice::from_mut(&mut self.buffer.buffer,),)?;
      self.buffer.reset();
    }

    Ok(self)
  }
  /// Clears the internal buffer, returning its current state.
  #[inline]
  pub fn clear_buffer(&mut self,) -> WriteByte {
    core::mem::replace(&mut self.buffer, WriteByte::EMPTY,)
  }
  /// Unwraps the inner writer if the inner buffer is empty.
  /// 
  /// If an unaligned error is returned, and the misalignment is `8` bits try flushing
  /// the writer, else write the missing bits.  
  pub fn into_writer(self,) -> Result<W, UnalignedError<Self,>> {
    match self.buffer.cursor {
      Some(Bits::B8) => Ok(self.writer),
      misalign => Err(UnalignedError(self, misalign.unwrap_or(Bits::B8,),)),
    }
  }
}

impl<W,> BitWrite for WriteIO<W,>
  where W: Write, {
  type Error = (Option<Bits>, Error,);

  #[inline]
  fn is_aligned(&self,) -> bool { self.buffer.cursor == Some(Bits::B8) }
  fn write_bits(&mut self, bits: Bits, buf: u8,) -> Result<Bits, Self::Error> {
    //Write the bits too the internal buffer.
    match self.buffer.write_bits(bits, buf,) {
      //Attempt to flush the inner buffer.
      Ok(bits) => { self.flush().map_err(|e,| (None, e,),)?; Ok(bits) },
      //Attempt to flush the inner buffer and continue writing.
      Err(to_write) => self.flush()
        //Return any error while flushing.
        .map_err(move |e,| (Some(to_write), e,),)?
        //Continue writing.
        .write_bits(to_write, buf,),
    }
  }
}

impl<W,> Read for WriteIO<W,>
  where W: Read + Write, {
  #[inline]
  fn read(&mut self, buf: &mut [u8],) -> io::Result<usize> { W::read(&mut self.writer, buf,) }
}
