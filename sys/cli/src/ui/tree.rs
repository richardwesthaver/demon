use cursive::traits::*;
pub use cursive_tree_view::{Placement, TreeView};
use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TreeEntry {
  pub name: String,
  pub dir: Option<PathBuf>,
}

impl fmt::Display for TreeEntry {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

pub fn collect_entries(dir: &PathBuf, entries: &mut Vec<TreeEntry>) -> io::Result<()> {
  if dir.is_dir() {
    for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() {
        entries.push(TreeEntry {
          name: entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| "".to_string()),
          dir: Some(path),
        });
      } else if path.is_file() {
        entries.push(TreeEntry {
          name: entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| "".to_string()),
          dir: None,
        });
      }
    }
  }
  Ok(())
}

pub fn expand_tree(tree: &mut TreeView<TreeEntry>, parent_row: usize, dir: &PathBuf) {
  let mut entries = Vec::new();
  collect_entries(dir, &mut entries).ok();

  entries.sort_by(|a, b| match (a.dir.is_some(), b.dir.is_some()) {
    (true, true) | (false, false) => a.name.cmp(&b.name),
    (true, false) => Ordering::Less,
    (false, true) => Ordering::Greater,
  });

  for i in entries {
    if i.dir.is_some() {
      tree.insert_container_item(i, Placement::LastChild, parent_row);
    } else {
      tree.insert_item(i, Placement::LastChild, parent_row);
    }
  }
}
