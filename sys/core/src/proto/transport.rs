use std::{convert::TryInto, fmt, num::TryFromIntError, sync::Arc, time::Duration};

use crate::{
  VarInt,
};

/// Core demon transport configuration
pub struct TransportConfig {
  pub(crate) stream_window_bidi: VarInt,
	pub(crate) stream_window_uni: VarInt,
	pub(crate) stream_receive_window: VarInt,
	pub(crate) receive_window: VarInt,
	pub(crate) send_window: u64,

	pub(crate) datagram_receive_buffer_size: Option<usize>,
	pub(crate) datagram_send_buffer_size: usize,
}

impl TransportConfig {
// TODO
}
