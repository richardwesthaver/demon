pub(crate) mod std;

use duckscript::types::{command::Commands, error::ScriptError};

pub(crate) fn load(commands: &mut Commands) -> Result<(), ScriptError> {
  std::load(commands)?;

  Ok(())
}
