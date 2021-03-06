use std::{fs, path::PathBuf, time::SystemTime};

use anyhow::Result;

use crate::{Entity, Source};

#[derive(Debug)]
pub(crate) struct FileSource {
  directory: PathBuf,
  extension: Option<String>,
}

impl FileSource {
  pub(crate) fn new(directory: PathBuf, extension: impl Into<Option<String>>) -> Self {
    Self {
      directory,
      extension: extension.into(),
    }
  }
}

impl Source for FileSource {
  fn config_for_path(&self, path: &str) -> Result<Entity> {
    let path = {
      let mut path_with_extension = path.to_owned();
      if let Some(extension) = &self.extension {
        path_with_extension.push_str(extension);
      }
      self.directory.join(path_with_extension)
    };

    let contents = fs::read_to_string(&path)?;
    let version = Some(contents.clone());

    let mod_time = fs::metadata(path)?
      .modified()?
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_secs();

    Ok(Entity {
      contents,
      mod_time,
      version,
    })
  }

  fn paths_to_refresh<'a>(&self, paths: &mut dyn Iterator<Item = &'a str>) -> Vec<&'a str> {
    paths.collect()
  }
}
