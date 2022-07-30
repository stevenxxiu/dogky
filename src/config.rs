extern crate xdg;

use std::error::Error;
use std::fs::File;

use gtk4::prelude::{DisplayExt, MonitorExt, NativeExt, WidgetExt};
use gtk4::ApplicationWindow;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone, Copy)]
pub struct ConfigProps {
  pub margin: u32,
  pub width: u32,
}

pub struct Config {
  pub config_props: ConfigProps,
  pub css_bytes: Vec<u8>,
}

impl Config {
  pub fn load() -> Result<Config, Box<dyn Error>> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("dogky").unwrap();

    let config_path = xdg_dirs.place_config_file("dogky.yaml")?;
    let config_file = File::open(config_path)?;
    let config_props: ConfigProps = serde_yaml::from_reader(config_file)?;

    let style_path = xdg_dirs.place_config_file("style.css")?;
    let css_bytes = std::fs::read(style_path).unwrap();

    Ok(Config {
      config_props,
      css_bytes,
    })
  }
}

impl ConfigProps {
  pub fn calculate_size(self: &Self, window: &ApplicationWindow) -> (u32, u32, u32, u32) {
    let surface = window.surface();
    let display = window.display();
    let monitor = display.monitor_at_surface(&surface);
    let monitor_width = monitor.geometry().width() as u32;
    let monitor_height = monitor.geometry().height() as u32;
    let window_width = self.width + 2 * self.margin;
    let window_height = monitor_height;
    (monitor_width, monitor_height, window_width, window_height)
  }
}
