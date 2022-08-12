extern crate xdg;

use std::error::Error;
use std::fs::File;

use gtk::prelude::{DisplayExt, MonitorExt, NativeExt, WidgetExt};
use gtk::Window;
use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::{SerializableCommand, SerializableRegex};

#[derive(Clone, Deserialize)]
pub struct WeatherProps {
  pub update_interval: u32,
  pub retry_timeout: u32,
  pub openweather_api_key: String,
  pub openweather_city_id: u64,
}

#[derive(Clone, Deserialize)]
pub struct CpuBarsProps {
  pub num_per_row: usize,
  pub width: u32,
  pub height: u32,
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Clone, Deserialize)]
pub struct CpuMemoryGraphProps {
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Clone, Deserialize)]
pub struct CpuMemoryGraphContainerProps {
  pub width: u32,
  pub height: u32,
  pub cpu: CpuMemoryGraphProps,
  pub memory: CpuMemoryGraphProps,
}

#[derive(Clone, Deserialize)]
pub struct CpuMemoryProcessListProps {
  pub num_processes: usize,
  pub pid_width: usize,
  pub cpu_width: usize,
  pub memory_width: usize,
  pub top_command: SerializableCommand,
}

#[derive(Clone, Deserialize)]
pub struct CpuMemoryProps {
  pub update_interval: u32,
  pub cpu_bars: CpuBarsProps,
  pub graphs: CpuMemoryGraphContainerProps,
  pub process_list: CpuMemoryProcessListProps,
}

#[derive(Clone, Deserialize)]
pub struct DiskBarProps {
  pub height: u32,
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Clone, Deserialize)]
pub struct DiskProps {
  pub update_interval: u32,
  pub name: String,
  pub device_path: String,
  pub mount_point: String,
  pub bar: DiskBarProps,
}

#[derive(Clone, Deserialize)]
pub struct GpuProps {
  pub update_interval: u32,
}

#[derive(Clone, Deserialize)]
pub struct NetworkGraphProps {
  pub border_color: String,
  pub fill_color: String,
  pub maximum_bytes_per_sec: u64,
}

#[derive(Clone, Deserialize)]
pub struct NetworkGraphContainerProps {
  pub width: u32,
  pub height: u32,
  pub upload: NetworkGraphProps,
  pub download: NetworkGraphProps,
}

#[derive(Clone, Deserialize)]
pub struct NetworkProps {
  pub update_interval: u32,
  pub public_ip_update_interval: Option<u32>,
  pub interface_regex: SerializableRegex,
  pub graphs: NetworkGraphContainerProps,
}

#[derive(Clone, Deserialize)]
pub struct ConfigProps {
  pub width: u32,
  pub weather: WeatherProps,
  pub cpu_memory: CpuMemoryProps,
  pub disk: DiskProps,
  pub gpu: GpuProps,
  pub network: NetworkProps,
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
