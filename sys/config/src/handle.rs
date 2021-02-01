use std::sync::Arc;

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::from_str;

use crate::refreshable_entities::RegisteredConfigEntity;

/// A configuration handle, with self-refresh if obtained from a `ConfigStore`.
/// If your type `T` implements `Default`, then this will implement `Default`
/// using a fixed config matching `T`'s default
#[derive(Clone)]
pub struct ConfigHandle<T> {
  inner: ConfigHandleImpl<T>,
}

// Enums have all their variants public, which needlessly exposes implementation
// details of the ConfigHandle, that is why this enum is wrapped in a struct.
#[derive(Clone)]
enum ConfigHandleImpl<T> {
  /// Config is obtained from a `ConfigStore`, and kept up to date
  Registered(Arc<RegisteredConfigEntity<T>>),
  /// Config is fixed. Obtained via `from_json`, `default` etc
  Fixed(Arc<T>),
}

impl<T> ConfigHandle<T>
where
  T: Send + Sync + 'static,
{
  /// Fetch the current version of the config referred to by this handle
  /// Return is an `Arc` so that if the config is updated after you get it, you
  /// will simply own an outdated pointer
  pub fn get(&self) -> Arc<T> {
    match &self.inner {
      ConfigHandleImpl::Registered(handle) => handle.get(),
      ConfigHandleImpl::Fixed(contents) => contents.clone(),
    }
  }

  pub(crate) fn from_registered(registered: Arc<RegisteredConfigEntity<T>>) -> Self {
    Self {
      inner: ConfigHandleImpl::Registered(registered),
    }
  }
}

impl<T> ConfigHandle<T>
where
  T: Send + Sync + DeserializeOwned + 'static,
{
  /// Create a static config handle from a JSON blob. Useful for testing.
  pub fn from_json(data: &str) -> Result<Self> {
    Ok(Self {
      inner: ConfigHandleImpl::Fixed(Arc::new(from_str(data)?)),
    })
  }
}

impl<T> Default for ConfigHandle<T>
where
  T: Send + Sync + Default + 'static,
{
  fn default() -> Self {
    Self {
      inner: ConfigHandleImpl::Fixed(Arc::new(T::default())),
    }
  }
}
