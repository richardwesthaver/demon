use duckscript::types::command::{Command, CommandResult};

use crate::utils::pckg;
#[cfg(test)]
#[path = "./test.rs"]
mod mod_test;

#[derive(Clone)]
pub(crate) struct CommandImpl {
  package: String,
}

impl Command for CommandImpl {
  fn name(&self) -> String {
    pckg::concat(&self.package, "Echo")
  }

  fn aliases(&self) -> Vec<String> {
    vec!["echo".to_string()]
  }

  fn help(&self) -> String {
    include_str!("help.md").to_string()
  }

  fn clone_and_box(&self) -> Box<dyn Command> {
    Box::new((*self).clone())
  }

  fn run(&self, arguments: Vec<String>) -> CommandResult {
    for argument in &arguments {
      print!("{} ", argument);
    }

    println!();

    CommandResult::Continue(Some(arguments.len().to_string()))
  }
}

pub(crate) fn create(package: &str) -> Box<dyn Command> {
  Box::new(CommandImpl {
    package: package.to_string(),
  })
}
