/// Status of this service.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DemonStatus {
  /// Service is dead
  Dead,
  /// Service is starting
  Starting,
  /// Service is alive and well
  Alive,
  /// Service is in the process of stopping
  Stopping,
  /// Service is already stopped
  Stopped,
  /// Service is alive, but something is wrong with it
  Warning,
}
