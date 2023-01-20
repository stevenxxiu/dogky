use std::path::{Path, PathBuf};

use gtk::gio;

fn get_output_path() -> PathBuf {
  // <root or manifest path>/target/<profile>/
  let manifest_dir_str = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let build_type = std::env::var("PROFILE").unwrap();
  let path = Path::new(&manifest_dir_str).join("target").join(build_type);
  return PathBuf::from(path);
}

fn compile_gtk_resources(cur_dir: &PathBuf) {
  gio::compile_resources(
    cur_dir.join("src/resources"),
    cur_dir.join("src/resources/resources.gresource.xml").to_str().unwrap(),
    "dogky.gresource",
  );
}

fn compile_sass(cur_dir: &PathBuf) {
  let src_path = cur_dir.join("src/resources/style.sass");
  let dest_path = cur_dir.join("src/resources/style.css");
  let sass_str = sass_rs::compile_file(
    src_path,
    sass_rs::Options {
      // Don't use `Compressed`, or *GTK* will warn that we need a `;` at the end of the block
      output_style: sass_rs::OutputStyle::Compact,
      precision: 1,
      indented_syntax: true,
      include_paths: vec![],
    },
  )
  .unwrap();
  std::fs::write(dest_path, sass_str).unwrap();
}

fn copy_scripts(cur_dir: &PathBuf) {
  let target_dir = get_output_path();
  let src = Path::join(&cur_dir, "src/move_window.py");
  let dest = Path::join(Path::new(&target_dir), Path::new("move_window.py"));
  std::fs::copy(src, dest).unwrap();
}

fn main() {
  let cur_dir = std::env::current_dir().unwrap();
  compile_gtk_resources(&cur_dir);
  copy_scripts(&cur_dir);
  compile_sass(&cur_dir);
}
