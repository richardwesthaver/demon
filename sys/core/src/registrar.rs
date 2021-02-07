// Demon Registrar Service
//
// This module provides a cached `Registry` for local usage by peers.
// The Registry is a HashMap<K: V> where each Entry contains an ID and Weight.
use tokio::sync::{broadcast, Notify};
use tokio::time::{self, Duration, Instant};
use bytes::Bytes;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

/// Registrar state shared across all local peer connections.
///
/// The 'Registrar' contains a 'HashMap' storing the key/value data and all
/// 'broadcast::Sender' values for active pub/sub channels
///
/// When a `Registrar` entry is created, a background task is spawned. This task is
/// used to expire values after the requested duration has elapsed. The task
/// runs until all instances of `Registrar` are dropped, at which point the task
/// terminates completely.
#[derive(Debug, Clone)]
pub struct Registrar {
    /// Handle to shared state. The background task will also have an
    /// `Arc<Shared>`.		
		shared: Arc<Shared>,
}

impl Registrar {
		pub fn new() -> Registrar {
				let shared = Arc::new(Shared {
						state: Mutex::new(State {
								entries: HashMap::new(),
								pub_sub: HashMap::new(),
								expirations: BTreeMap::new(),
								next_id: 0,
								shutdown: false,
						}),
						background_task: Notify::new(),
				});
        // Start the background task.
//        tokio::spawn(purge_expired_tasks(shared.clone()));
				Registrar { shared }
		}

		/// Get the value associated with a key
		///
		/// Returns 'None' if there is no value currently assigned.
		pub fn get(&self, key:&str) -> Option<Bytes> {
				// acquire the state lock, get entry by key, clone associated
				// value
				let state = self.shared.state.lock().unwrap();
				state.entries.get(key).map(|entry| entry.data.clone())
		}

		/// Set the value of a specified key with an optional expiration
		///
		/// If a value is already associated with the given key, it is replaced.
		pub(crate) fn set(&self, key: String, value: Bytes, expires: Option<Duration>) {
				let mut state = self.shared.state.lock().unwrap();

				// Get and increment the next entry ID. This guarantees a Unique ID associated with each 'set' operation.
				let id = state.next_id;
				state.next_id += 1;

				// don't notify just yet
        let mut notify = false;

				let expires = expires.map(|duration| {
						// Instant at which the key expires
						let when = Instant::now() + duration;

						// Only notify the worker task if the newly inserted expiration is the **next** key to evict. Implies that the worker needs to be awoken.
						notify = state.next_expiration().map(|expiration| expiration > when)
								.unwrap_or(true);

						// Track the expiration
						state.expirations.insert((when, id), key.clone());
						when
				});

				// Insert the entry
				let prev = state.entries.insert(
						key,
						Entry {
								id,
								data: value,
								expires,
						},
				);

        // If there was a value previously associated with the key **and** it
        // had an expiration time. The associated entry in the `expirations` map
        // must also be removed. This avoids leaking data.
				if let Some(prev) = prev {
						if let Some(when) = prev.expires {
								// clear expiration
								state.expirations.remove(&(when, prev.id));
						}
				}

				// Release the lock before notifying the background task to reduce contention
				drop(state);

				if notify {
						// only notify the background task if it needs to update its state with a new expiration
						self.shared.background_task.notify_one();
				}
		}

		/// Returns a `Receiver` for the requested channel.
    pub(crate) fn subscribe(&self, key: String) -> broadcast::Receiver<Bytes> {
        use std::collections::hash_map::Entry;

        // Acquire the mutex
        let mut state = self.shared.state.lock().unwrap();

        // If there is no entry for the requested channel, then create a new
        // broadcast channel and associate it with the key. If one already
        // exists, return an associated receiver.
        match state.pub_sub.entry(key) {
            Entry::Occupied(e) => e.get().subscribe(),
            Entry::Vacant(e) => {
                // No broadcast channel exists yet, so create one.
                //
                // When the channel's capacity fills up, publishing will result
                // in old messages being dropped. This prevents slow consumers
                // from blocking the entire system.
                let (tx, rx) = broadcast::channel(1024);
                e.insert(tx);
                rx
            }
        }
    }

    /// Publish a message to the channel. Returns the number of subscribers
    /// listening on the channel.
    pub(crate) fn publish(&self, key: &str, value: Bytes) -> usize {
        let state = self.shared.state.lock().unwrap();

        state
            .pub_sub
            .get(key)
            // On a successful message send on the broadcast channel, the number
            // of subscribers is returned. An error indicates there are no
            // receivers, in which case, `0` should be returned.
            .map(|tx| tx.send(value).unwrap_or(0))
            .unwrap_or(0)
    }		
}

impl Drop for Registrar {
    fn drop(&mut self) {
        // If this is the last active `Registrar` instance, the background task must be
        // notified to shut down.
        //
        // First, determine if this is the last `Registrar` instance. This is done by
        // checking `strong_count`. The count will be 2. One for this `Registrar`
        // instance and one for the handle held by the background task.
        if Arc::strong_count(&self.shared) == 2 {
            // The background task must be signaled to shutdown. This is done by
            // setting `State::shutdown` to `true` and signalling the task.
            let mut state = self.shared.state.lock().unwrap();
            state.shutdown = true;

            // Drop the lock before signalling the background task. This helps
            // reduce lock contention by ensuring the background task doesn't
            // wake up only to be unable to acquire the mutex.
            drop(state);
            self.shared.background_task.notify_one();
        }
    }
}

#[derive(Debug)]
struct Shared {
		state: Mutex<State>,
    /// Notifies the background task handling entry expiration. The background
    /// task waits on this to be notified, then checks for expired values or the
    /// shutdown signal.
    background_task: Notify,		
}


impl Shared {
    /// Purge all expired keys and return the `Instant` at which the **next**
    /// key will expire. The background task will sleep until this instant.		
		fn purge_expired(&self) -> Option<Instant> {
				let mut state = self.state.lock().unwrap();

				if state.shutdown {
						// The database is shutting down, the background task should exit
						return None;
				}

				let state = &mut *state;
        // Find all keys scheduled to expire **before** now.
				let now = Instant::now();

        while let Some((&(when, id), key)) = state.expirations.iter().next() {
            if when > now {
                // Done purging, `when` is the instant at which the next key
                // expires. The worker task will wait until this instant.
                return Some(when);
            }

            // The key expired, remove it
            state.entries.remove(key);
            state.expirations.remove(&(when, id));
        }

        None				
		}

		/// Returns 'true' if the registry is shutting down
		///
		/// The flag is set when all 'Registrar' values have dropped
		fn is_shutdown(&self) -> bool {
				self.state.lock().unwrap().shutdown
		}
}

#[derive(Debug)]
struct State {
		/// The key-value data
		entries: HashMap<String, Entry>,
		pub_sub: HashMap<String, broadcast::Sender<Bytes>>,
    expirations: BTreeMap<(Instant, u64), String>,
    next_id: u64,
    shutdown: bool,
}

impl State {
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .keys()
            .next()
            .map(|expiration| expiration.0)
    }
}

/// A single Entry in the registry
#[derive(Debug)]
struct Entry {
		/// UID
		id: u64,
		/// Stored data
		data: Bytes,
		/// Instant at which the entry expires and should be removed from
		/// the registry
		expires: Option<Instant>,
}

/// Routine executed by the background task.
///
/// Wait to be notified. On notification, purge any expired keys from the shared
/// state handle. If `shutdown` is set, terminate the task.
async fn purge_expired_tasks(shared: Arc<Shared>) {
    // If the shutdown flag is set, then the task should exit.
    while !shared.is_shutdown() {
        // Purge all keys that are expired. The function returns the instant at
        // which the **next** key will expire. The worker should wait until the
        // instant has passed then purge again.
        if let Some(when) = shared.purge_expired() {
            // Wait until the next key expires **or** until the background task
            // is notified. If the task is notified, then it must reload its
            // state as new keys have been set to expire early. This is done by
            // looping.
            tokio::select! {
                _ = time::sleep_until(when) => {}
                _ = shared.background_task.notified() => {}
            }
        } else {
            // There are no keys expiring in the future. Wait until the task is
            // notified.
            shared.background_task.notified().await;
        }
    }
}
