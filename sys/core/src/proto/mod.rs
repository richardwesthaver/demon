#[doc(hidden)]
pub mod coding;

mod varint;
pub use varint::{VarInt, VarIntBoundsExceeded};

mod shared;
use std::{fmt, net::SocketAddr, ops};

pub use shared::{ConnectionEvent, ConnectionId, EndpointEvent};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Side {
  /// The initiator of a connection
  Client = 0,
  /// The acceptor of a connection
  Server = 1,
}

impl Side {
  #[inline]
  /// Shorthand for `self == Side::Client`
  pub fn is_client(self) -> bool {
    self == Side::Client
  }

  #[inline]
  /// Shorthand for `self == Side::Server`
  pub fn is_server(self) -> bool {
    self == Side::Server
  }
}

impl ops::Not for Side {
  type Output = Side;
  fn not(self) -> Side {
    match self {
      Side::Client => Side::Server,
      Side::Server => Side::Client,
    }
  }
}

/// Whether a stream communicates data in both directions or only from the
/// initiator
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Dir {
  /// Data flows in both directions
  Bi = 0,
  /// Data flows only from the stream's initiator
  Uni = 1,
}

impl Dir {
  fn iter() -> impl Iterator<Item = Self> {
    [Dir::Bi, Dir::Uni].iter().cloned()
  }
}

impl fmt::Display for Dir {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use self::Dir::*;
    f.pad(match *self {
      Bi => "bidirectional",
      Uni => "unidirectional",
    })
  }
}

/// Identifier for a stream within a particular connection
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StreamId(#[doc(hidden)] pub u64);

impl fmt::Display for StreamId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let initiator = match self.initiator() {
      Side::Client => "client",
      Side::Server => "server",
    };
    let dir = match self.dir() {
      Dir::Uni => "uni",
      Dir::Bi => "bi",
    };
    write!(
      f,
      "{} {}directional stream {}",
      initiator,
      dir,
      self.index()
    )
  }
}

impl StreamId {
  /// Create a new StreamId
  pub fn new(initiator: Side, dir: Dir, index: u64) -> Self {
    StreamId(index << 2 | (dir as u64) << 1 | initiator as u64)
  }
  /// Which side of a connection initiated the stream
  pub fn initiator(self) -> Side {
    if self.0 & 0x1 == 0 {
      Side::Client
    } else {
      Side::Server
    }
  }
  /// Which directions data flows in
  pub fn dir(self) -> Dir {
    if self.0 & 0x2 == 0 {
      Dir::Bi
    } else {
      Dir::Uni
    }
  }
  /// Distinguishes streams of the same initiator and directionality
  pub fn index(self) -> u64 {
    self.0 >> 2
  }
}

impl coding::Codec for StreamId {
  fn decode<B: bytes::Buf>(buf: &mut B) -> coding::Result<StreamId> {
    VarInt::decode(buf).map(|x| StreamId(x.into_inner()))
  }
  fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
    VarInt::from_u64(self.0).unwrap().encode(buf);
  }
}

/// An outgoing packet
#[derive(Debug)]
pub struct Transmit {
  /// The socket this datagram should be sent to
  pub destination: SocketAddr,
  /// Contents of the datagram
  pub contents: Vec<u8>,
}
