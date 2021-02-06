use std::{ops::RangeFrom, sync::Mutex};

use lazy_static::lazy_static;

lazy_static! {
  pub static ref SERVER_PORTS: Mutex<RangeFrom<u16>> = Mutex::new(8888..);
  pub static ref CLIENT_PORTS: Mutex<RangeFrom<u16>> = Mutex::new(32328..);
}
