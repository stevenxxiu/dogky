extern crate xdg;

use std::error::Error;
use std::fs::File;

use gtk::prelude::{DisplayExt, MonitorExt, NativeExt, WidgetExt};
use serde_derive::{Deserialize, Serialize};

use crate::components::Window;
use crate::path::get_xdg_dirs;

#[derive(Serialize, Deserialize)]
pub struct WeatherProps {
  pub update_interval: u32,
  pub retry_timeout: u32,
  pub openweather_api_key: String,
  pub openweather_city_id: u64,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigProps {
  pub width: u32,
  pub weather: WeatherProps,
}

pub struct Config {
  pub config_props: ConfigProps,
  pub css_bytes: Option<Vec<u8>>,
}

impl Config {
  pub fn load() -> Result<Config, Box<dyn Error>> {
    let config_path = get_xdg_dirs().place_config_file("dogky.yaml")?;
    let config_file = File::open(config_path)?;
    let config_props: ConfigProps = serde_yaml::from_reader(config_file)?;

    let style_path = get_xdg_dirs().place_config_file("style.css")?;
    let css_bytes = std::fs::read(style_path).ok();

    Ok(Config {
      config_props,
      css_bytes,
    })
  }
}

impl ConfigProps {
  pub fn calculate_size(self: &Self, window: &Window) -> (u32, u32, u32, u32) {
    let surface = window.surface();
    let display = window.display();
    let monitor = display.monitor_at_surface(&surface);
    let monitor_width = monitor.geometry().width() as u32;
    let monitor_height = monitor.geometry().height() as u32;
    let window_width = self.width;
    let window_height = monitor_height;
    (monitor_width, monitor_height, window_width, window_height)
  }
}
