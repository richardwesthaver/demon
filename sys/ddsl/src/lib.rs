pub(crate) mod sdk;
#[cfg(test)]
mod test;
mod utils;
use std::env;

pub use duckscript::{
  runner,
  types::{command::Commands, error::ScriptError, runtime::Context},
};
pub use duckscriptsdk;

static VERSION: &str = env!("CARGO_PKG_VERSION");
static AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
static DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Returns the library version.
pub fn version() -> String {
  VERSION.to_string()
}

/// Loads all core commands
pub fn load(commands: &mut Commands) -> Result<(), ScriptError> {
  sdk::load(commands)
}

pub fn create_context() -> Result<Context, ScriptError> {
  let mut context = Context::new();
  duckscriptsdk::load(&mut context.commands)?;

  Ok(context)
}

pub fn run_script(value: &str, is_file: bool) -> Result<(), ScriptError> {
  let context = create_context()?;

  if is_file {
    runner::run_script_file(value, context)?;
  } else {
    runner::run_script(value, context)?;
  }

  Ok(())
}

pub fn run_repl() -> Result<(), ScriptError> {
  let context = create_context()?;

  runner::repl(context)?;

  Ok(())
}
