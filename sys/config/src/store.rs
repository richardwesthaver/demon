use std::{
  collections::HashMap,
  fmt,
  path::PathBuf,
  sync::{Arc, Condvar, Mutex, Weak},
  thread,
  time::Duration,
};

use anyhow::Result;
use serde::de::DeserializeOwned;
use slog::{info, warn, Logger};

use crate::{
  file_source::FileSource,
  handle::ConfigHandle,
  refreshable_entities::{Refreshable, RegisteredConfigEntity},
  Source,
};

/// A wrapper around the configerator APIs to provide an easily mocked way of
/// reading JSON configs into Serde-compatible structures.
#[derive(Clone)]
pub struct ConfigStore {
  source: Arc<dyn Source + Sync + Send>,
  clients: Arc<Mutex<HashMap<String, ClientList>>>,
  kick: Arc<Condvar>,
  logger: Option<Logger>,
}

type ClientList = Vec<Weak<dyn Refreshable + Sync + Send>>;

impl ConfigStore {
  /// Create a new instance of the ConfigStore with its own updating thread
  /// which will be run every `poll_interval`. The configs will be retrieved
  /// from the provided `source`. If `logger` is given then the store will
  /// inform about status of refreshes. If `poll_interval` is None then no
  /// updating thread will be spawned.
  ///
  /// TODO: Each instance creates its own thread, make sure the thread is
  /// stopped once the ConfigStore and all relevant ConfigHandle are destroyed.
  pub fn new(
    source: Arc<dyn Source + Sync + Send>,
    poll_interval: impl Into<Option<Duration>>,
    logger: impl Into<Option<Logger>>,
  ) -> Self {
    let this = Self {
      source,
      clients: Arc::new(Mutex::new(HashMap::new())),
      kick: Arc::new(Condvar::new()),
      logger: logger.into(),
    };

    if let Some(poll_interval) = poll_interval.into() {
      thread::Builder::new()
        .name("rust-cfgr-updates".into())
        .spawn({
          let this = this.clone();
          move || this.updater_thread(poll_interval)
        })
        .expect("Can't spawn cached_config updates poller");
    }

    this
  }

  /// Get configs from JSON files on disk.
  /// `logger` is `None` if no desire to log from the background thread, or a
  /// `slog::Logger` to log to. `prefix` is the directory prefix to apply to
  /// all config paths to find the on-disk JSON `suffix` is a file suffix to
  /// add to get the config JSON `poll_interval` is the sleep time between
  /// checks for config changes
  pub fn file(
    logger: impl Into<Option<Logger>>,
    directory: PathBuf,
    extension: impl Into<Option<String>>,
    poll_interval: impl Into<Option<Duration>>,
  ) -> Self {
    Self::new(
      Arc::new(FileSource::new(directory, extension)),
      poll_interval,
      logger.into(),
    )
  }

  /// Fetch a self-updating config handle for the config at `path`.
  /// See `ConfigHandle` for uses of this handle.
  pub fn get_config_handle<T>(&self, path: String) -> Result<ConfigHandle<T>>
  where
    T: Send + Sync + DeserializeOwned + 'static,
  {
    fn deserialize_json<T: DeserializeOwned>(s: String) -> Result<T> {
      let v = serde_json::from_str(&s)?;
      Ok(v)
    }
    self.get_config_handle_with_deserializer(path, deserialize_json)
  }

  /// Fetch a self-updating config handle for the config at `path`, as a raw,
  /// non-deserialized string. This is usually not what you want if you need
  /// to use the config (since you won't get the benefits of a cached
  /// deserialization), so prefer using `get_config_handle`. That said, if you
  /// need to pass the config through to something else, this is the method you
  /// want.
  pub fn get_raw_config_handle(&self, path: String) -> Result<ConfigHandle<String>> {
    fn deserialize_raw(s: String) -> Result<String> {
      Ok(s)
    }
    self.get_config_handle_with_deserializer(path, deserialize_raw)
  }

  fn get_config_handle_with_deserializer<T>(
    &self,
    path: String,
    deserializer: fn(String) -> Result<T>,
  ) -> Result<ConfigHandle<T>>
  where
    T: Send + Sync + 'static,
  {
    let entity = {
      let entity = self.source.config_for_path(&path)?;
      Arc::new(RegisteredConfigEntity::new(
        path.clone(),
        entity,
        deserializer,
      )?)
    };

    let mut clients = self.clients.lock().expect("lock poisoned");

    let client_handle = clients.entry(path).or_insert_with(Vec::new);
    client_handle.push(Arc::downgrade(&entity) as Weak<dyn Refreshable + Send + Sync>);

    self.kick.notify_one();

    Ok(ConfigHandle::from_registered(entity))
  }

  fn refresh_client(&self, client: Arc<dyn Refreshable + Sync + Send>) {
    let res = self
      .source
      .config_for_path(client.get_path())
      .and_then(|entity| client.refresh(entity));
    if let Some(ref logger) = self.logger {
      match res {
        Ok(false) => {}
        Ok(true) => info!(logger, "Updated path {}", client.get_path()),
        Err(e) => warn!(
          logger,
          "Failed to update path {} due to {:#?}",
          client.get_path(),
          e
        ),
      }
    }
  }

  fn refresh_client_list(&self, client_list: &[Weak<dyn Refreshable + Sync + Send>]) {
    for client in client_list {
      if let Some(client) = client.upgrade() {
        self.refresh_client(client);
      }
    }
  }

  fn updater_thread(&self, poll_interval: Duration) {
    loop {
      let clients = self.clients.lock().expect("lock poisoned");

      // Don't loop when there are no active clients to care about
      let mut clients = if clients.is_empty() {
        self.kick.wait(clients).expect("Lock poisoned")
      } else {
        clients
      };

      for path in self
        .source
        .paths_to_refresh(&mut clients.keys().map(|x| -> &str { x }))
      {
        if let Some(client_list) = clients.get(path) {
          self.refresh_client_list(client_list);
        }
      }

      // Remove lost clients
      clients.retain(|_, client_list| {
        // Remove all vanished clients from the list, then check if it's empty
        client_list.retain(|client| client.upgrade().is_some());
        !client_list.is_empty()
      });

      // Release clients before going to sleep.
      std::mem::drop(clients);

      thread::sleep(poll_interval);
    }
  }
}

impl fmt::Debug for ConfigStore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "ConfigStore({:?})", self.source)
  }
}

#[cfg(test)]
mod test {
  use std::time::Instant;

  use anyhow::Error;

  use super::*;
  use crate::TestSource;

  #[test]
  fn test_contention() -> Result<(), Error> {
    let source = TestSource::new();
    source.insert_config("foo", "bar", 0);

    let store = ConfigStore::new(Arc::new(source), Some(Duration::from_millis(100)), None);

    // Now, acquire a handle. This will let the updater go to sleep instead of
    // waiting on a condition variable.
    let h = store.get_raw_config_handle("foo".to_string())?;
    thread::sleep(Duration::from_millis(200));

    // Now, try to acquire handles. If the updater released the lock properly, this
    // will work. Otherwise, we might have to wait 90ms (100 - 10) until it
    // releases, or forever if it just re-acquires the lock immediately.
    let t0 = Instant::now();
    store.get_raw_config_handle("foo".to_string())?;
    store.get_raw_config_handle("foo".to_string())?;
    assert!(t0.elapsed().as_millis() < 10);

    // Drop the handle. We do this explicitly to make sure it does not get dropped
    // earlier.
    std::mem::drop(h);

    Ok(())
  }
}
