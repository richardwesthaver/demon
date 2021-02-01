// based on facebook/shed configerator
mod file_source;
mod handle;
mod refreshable_entities;
mod store;
mod test_source;
#[cfg(test)]
mod tests;

use std::{
  error::Error,
  fmt,
  fmt::Debug,
  io,
  path::{Path, PathBuf},
};

use anyhow::Result;
pub use handle::ConfigHandle;
use serde::{Deserialize, Serialize};
pub use store::ConfigStore;
pub use test_source::TestSource;

/// Trait to be implemented by sources of configuration that the `ConfigStore`
/// will use
pub trait Source: Debug {
  /// For a given path identifying the config return it's content
  fn config_for_path(&self, path: &str) -> Result<Entity>;
  /// Given a list of paths the client is interested in, return the ones that
  /// should be refreshed since the client last asked for them.
  fn paths_to_refresh<'a>(&self, paths: &mut dyn Iterator<Item = &'a str>) -> Vec<&'a str>;
}

/// Represents a configuration Entity e.g. a JSON blob
#[derive(Clone, Debug)]
pub struct Entity {
  /// Content of the config
  pub contents: String,
  /// Modification time of the config, e.g. file modification time
  pub mod_time: u64,
  /// Optional version of the config, together with mod_time it is used to
  /// decide if the config has changed or not
  pub version: Option<String>,
}
