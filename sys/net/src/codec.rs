
use tokio::net::UdpSocket;
use tokio_util::{Decoder, Encoder, LinesCodec};
use tokio_util::udp::UdpFramed;
use bytes::{BufMut, BytesMut};
use thiserror::Error;

use std::io;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Type(u64);

pub struct DemonCodec;

