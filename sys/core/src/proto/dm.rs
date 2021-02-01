// see https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/codec/length_delimited.rs
use tokio_util::codec::{Decoder, Encoder};
use bytes::{BytesMut, Buf};

struct Dm {}

const MAX: usize = 8 * 1024 * 1024;

impl Decoder for Dm {
  type Item = String;
  type Error = std::io::Error;

  fn decode(
    &mut self,
    src: &mut BytesMut
  ) -> Result<Option<Self::Item>, Self::Error> {
    if src.len() < 4 {
      // could not read frame header
      return Ok(None);
    }

    // Read header length_bytes
    let mut length_bytes = [0u8; 4];
    length_bytes.copy_from_slice(&src[..4]);
    let length = u64::from_le_bytes(length_bytes) as usize;

    if length > MAX {
      return Err(std::io::Error::new(
	std::io::ErrorKind::InvalidData,
	format!("Frame of length {} is too jumbo!!", length)
      ));
    }

    if src.len() < 4 + length {
      // Full string has not yet arrived.
      //
      // here we warm it up a bit.. good idea performance-wise.
      src.reserve(4 + length - src.len());

      return Ok(None);
    }

    let data = src[4..4 + length].to_vec();
    src.advance(4 + length);

    // Convert to string or fail if not valid utf-8.
    match String::from_utf8(data) {
      Ok(string) => Ok(Some(string)),
      Err(utf8_error) => {
	Err(std::io::Error::new(
	  std::io::ErrorKind::InvalidData,
	  utf8_error.utf8_error(),
	))
      },
    }
  }
}


impl Encoder for Dm {
  type Error = std::io::Error;

  fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
    // Don't send if the string is too long..
    if item.len() > MAX {
      return Err(std::io::Error::new(
	std::io::ErrorKind::InvalidData,
	format!("Frame of length {} is too large.", item.len())
      ));
    }

    // Convert header into byte array.
    let len_slice = u64::to_le_bytes(item.len() as u64);

    // Reserve space early
    dst.reserve(4 + item.len());

    // write the header and payload to the buffer.
    dst.extend_from_slice(&len_slice);
    dst.extend_from_slice(item.as_bytes());
    Ok(())
  }
}
