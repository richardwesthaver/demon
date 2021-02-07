mod proto;
pub use proto::VarInt;
mod circular_buffer;
mod controller;
#[cfg(feature = "macros")]
mod init;
mod range_set;
mod registrar;

#[cfg(test)]
mod tests;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
