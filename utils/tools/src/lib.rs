pub mod ascii_ext;
/// A collection of useful utilities
pub mod hostname;
pub mod net;
pub mod os;
pub mod timing;

use std::collections::HashMap;

use crossterm::event::Event;
pub use failure;
pub use ipnetwork;
pub use pnet;
use pnet::datalink::{DataLinkReceiver, NetworkInterface};
pub use tokio;

pub use crate::net::{dns, sniffer::*, LocalSocket};

pub struct OpenSockets {
  pub sockets_to_procs: HashMap<LocalSocket, String>,
}

pub struct OsInputOutput {
  pub network_interfaces: Vec<NetworkInterface>,
  pub network_frames: Vec<Box<dyn DataLinkReceiver>>,
  pub get_open_sockets: fn() -> OpenSockets,
  pub terminal_events: Box<dyn Iterator<Item = Event> + Send>,
  pub dns_client: Option<dns::Client>,
  pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}
