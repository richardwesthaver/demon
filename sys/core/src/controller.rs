// Core `Controller` trait
//
// Provides a remote-control API for Demons.
// The API provides human-readable checks on the underlying Resources allocated by a Demon,
// as well as an interface for `INIT' and 'KILL' commands.
use std::collections::HashMap;
use tokio::net::UdpSocket;
use tokio::{io, time};


pub struct Controller {
  socket: UdpSocket,
  command: u8,
  config: HashMap<u16, String>,
  rx_buf: Box<[u8]>,
  tx_buf: Box<[u8]>,
  
}

pub struct ControllerConfig;
