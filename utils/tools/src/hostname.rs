use std::io;

/// Returns hostname as reported by the system
pub fn get_hostname() -> io::Result<String> {
  Ok(hostname::get()?.to_string_lossy().into_owned())
}

/// Set system hostname to String specified
pub fn set_hostname(name: &str) -> io::Result<()> {
  hostname::set(name)?;
  Ok(())
}
