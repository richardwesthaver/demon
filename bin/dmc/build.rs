use std::{env, fs};
use vergen::{self, ConstantsFlags};

fn main() {
  // compile only with local build
  if Ok("dev".to_owned()) == env::var("PROFILE") {
    build_demon_dirs();
  }

  vergen::generate_cargo_keys(ConstantsFlags::all())
    .unwrap_or_else(|e| panic!("Vergen crate failed to generate version information! {}", e));

  println!("cargo:rerun-if-changed=build.rs");
}

/// Build any non-existing directories required
fn build_demon_dirs() {
  let demon_dirs = directories_next::ProjectDirs::from("io", "demon", "demon").unwrap();
  if !demon_dirs.config_dir().exists() {
    fs::create_dir(&demon_dirs.config_dir()).expect("could not create demon config directory.");
  }
  if !demon_dirs.data_local_dir().exists() {
    fs::create_dir(&demon_dirs.data_local_dir())
      .expect("could not create demon local data directory.");
  }
}
