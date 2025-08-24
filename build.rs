use std::path::{Path, PathBuf};

fn get_output_path() -> PathBuf {
  // <root or manifest path>/target/<profile>/
  let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let build_type = std::env::var("PROFILE").unwrap();
  Path::new(&manifest_dir_str).join("target").join(build_type)
}

fn copy_scripts(cur_dir: &Path) {
  let target_dir = get_output_path();
  let src = Path::join(cur_dir, "src/move_window.py");
  let dest = Path::join(Path::new(&target_dir), Path::new("move_window.py"));
  std::fs::copy(src, dest).unwrap();
}

fn main() {
  let cur_dir = std::env::current_dir().unwrap();
  copy_scripts(&cur_dir);
}
