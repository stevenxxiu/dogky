pub fn get_xdg_dirs() -> xdg::BaseDirectories {
  xdg::BaseDirectories::with_prefix("dogky")
}
