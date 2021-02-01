use std::sync::mpsc;

use cpal::{
  self,
  traits::{DeviceTrait, HostTrait, StreamTrait},
};
use dasp::{signal, Sample, Signal};

mod engine;
