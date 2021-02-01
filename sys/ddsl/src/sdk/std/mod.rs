mod echo;

use duckscript::types::{command::Commands, error::ScriptError};

static PACKAGE: &str = "std";

pub(crate) fn load(commands: &mut Commands) -> Result<(), ScriptError> {
  commands.set(echo::create(PACKAGE))?;
  Ok(())
}
