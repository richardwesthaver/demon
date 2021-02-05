//! A collection of structures and functions useful across the entire lib.
//#![feature(maybe_uninit_ref)]

pub mod proto;
pub use self::proto::VarInt;
mod circular_buffer;
mod controller;
mod range_set;
mod registrar;
#[cfg(test)]
mod tests;
