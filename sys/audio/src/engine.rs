// demon::audio I/O
use std::{collections::HashMap, fmt, io::BufReader, path::Path, thread, time::Duration};

use rodio::{self, dynamic_mixer, Source};

/// Number of playback channels.
const CHANNELS: u8 = 1;

/// Sample rate of playback.
const SAMPLE_RATE: u32 = 44_100;

/// Represents the playback tempo (beats per minute).
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Tempo(u16);

impl From<u16> for Tempo {
  #[inline]
  fn from(v: u16) -> Tempo {
    Tempo(v)
  }
}

impl fmt::Display for Tempo {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
