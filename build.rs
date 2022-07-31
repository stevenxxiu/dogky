use std::path::{Path, PathBuf};

use gtk::gio;

fn get_output_path() -> PathBuf {
  // <root or manifest path>/target/<profile>/
  let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let build_type = std::env::var("PROFILE").unwrap();
  let path = Path::new(&manifest_dir_str).join("target").join(build_type);
  return PathBuf::from(path);
}

fn main() {
  let cur_dir = std::env::current_dir().unwrap();
  gio::compile_resources(
    cur_dir.join("src/resources"),
    cur_dir.join("src/resources/resources.gresource.xml").to_str().unwrap(),
    "dogky.gresource",
  );

  let target_dir = get_output_path();
  let src = Path::join(&cur_dir, "src/move_window.sh");
  let dest = Path::join(Path::new(&target_dir), Path::new("move_window.sh"));
  std::fs::copy(src, dest).unwrap();
}
