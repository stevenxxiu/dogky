extern crate xdg;

use std::error::Error;
use std::fs::File;

use gtk4::prelude::{DisplayExt, MonitorExt, NativeExt, WidgetExt};
use gtk4::ApplicationWindow;
use serde_derive::Deserialize;

use crate::serializable_rgba::SerializableRGBA;

#[derive(Deserialize, Clone, Copy)]
pub struct Config {
  pub bg_color: SerializableRGBA,
}

impl Config {
  pub fn load() -> Result<Config, Box<dyn Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("dogky").unwrap();
    let config_path = xdg_dirs.place_config_file("dogky.yaml")?;
    let config_file = File::open(config_path)?;
    let config: Config = serde_yaml::from_reader(config_file)?;
    Ok(config)
  }

  pub fn calculate_size(self: &Self, window: &ApplicationWindow) -> (u32, u32) {
    let window_width = window.allocated_width() as u32;
    let surface = window.surface();
    let display = window.display();
    let monitor = display.monitor_at_surface(&surface);
    let window_height = monitor.geometry().height() as u32;
    (window_width, window_height)
  }
}
