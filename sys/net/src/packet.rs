use bytes::{Bytes, BytesMut};
use thiserror::Error;

pub(crate) struct Packet {
  pub(crate) header: Header,
  pub(crate) header_data: Bytes,
  pub(crate) payload: BytesMut,
}

pub struct Header {
  len: u64,
}

#[derive(Debug, Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub(crate) enum PacketDecodeError {
  #[error("invalid header: {0}")]
  InvalidHeader(&'static str),
}
