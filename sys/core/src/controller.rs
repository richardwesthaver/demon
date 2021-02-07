// Core `Controller` trait
//
// Provides a remote-control API for Demons.
// The API provides human-readable checks on the underlying Resources allocated by a Demon,
// as well as an interface for `INIT' and 'KILL' commands.
use tokio::sync::oneshot;
use bytes::Bytes;
use std::collections::HashMap;
use tokio::net::UdpSocket;
use crate::Result;

#[derive(Debug)]
pub enum Command {
		Get {
				key: String,
				rx: Responder<Option<Bytes>>,
		},
		Set {
				key: String,
				val: Bytes,
				rx: Responder<()>,
		},
}

type Responder<T> = oneshot::Sender<Result<T>>;

pub struct Controller {
  socket: UdpSocket,
  command: u8,
  config: HashMap<u16, String>,
  rx_buf: Box<[u8]>,
  tx_buf: Box<[u8]>,
}

pub struct ControllerConfig;
