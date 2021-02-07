pub mod proto;
pub use self::proto::VarInt;
mod circular_buffer;
mod controller;
mod range_set;
mod registrar;
#[cfg(test)]
mod tests;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
