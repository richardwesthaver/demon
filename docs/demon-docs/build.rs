use std::{env, fs, path::Path};

fn main() {
  let out_dir = env::var("OUT_DIR").unwrap();
  let dest_path = Path::new(&out_dir).join("file_path.txt");
}
