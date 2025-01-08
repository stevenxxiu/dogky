extern crate xdg;

use std::error::Error;
use std::fs::File;

use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::{SerializableCommand, SerializableRegex};

#[derive(Deserialize)]
pub struct WeatherProps {
  pub update_interval: u64,
  pub retry_timeout: u64,
  pub openweather_api_key: String,
  pub openweather_city_id: u64,
}

#[derive(Deserialize)]
pub struct CpuBarsProps {
  pub num_per_row: usize,
  pub width: f32,
  pub height: f32,
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Deserialize)]
pub struct CpuMemoryGraphProps {
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Deserialize)]
pub struct CpuMemoryGraphContainerProps {
  pub width: u32,
  pub height: u32,
  pub cpu: CpuMemoryGraphProps,
  pub memory: CpuMemoryGraphProps,
}

#[derive(Deserialize)]
pub struct CpuMemoryProcessListProps {
  pub num_processes: usize,
  pub pid_width: f32,
  pub cpu_width: f32,
  pub memory_width: f32,
  pub top_command: SerializableCommand,
}

#[derive(Deserialize)]
pub struct CpuMemoryProps {
  pub update_interval: u64,
  pub cpu_bars: CpuBarsProps,
  pub graphs: CpuMemoryGraphContainerProps,
  pub process_list: CpuMemoryProcessListProps,
}

#[derive(Deserialize)]
pub struct DiskBarProps {
  pub height: u32,
  pub border_color: String,
  pub fill_color: String,
}

#[derive(Deserialize)]
pub struct DiskProps {
  pub update_interval: u64,
  pub name: String,
  pub device_path: String,
  pub mount_point: String,
  pub bar: DiskBarProps,
}

#[derive(Deserialize)]
pub struct GpuProps {
  pub update_interval: u64,
}

#[derive(Deserialize)]
pub struct NetworkGraphProps {
  pub border_color: String,
  pub fill_color: String,
  pub maximum_bytes_per_sec: u64,
}

#[derive(Deserialize)]
pub struct NetworkGraphContainerProps {
  pub width: u32,
  pub height: u32,
  pub upload: NetworkGraphProps,
  pub download: NetworkGraphProps,
}

#[derive(Deserialize)]
pub struct NetworkProps {
  pub update_interval: u32,
  pub public_ip_update_interval: Option<u32>,
  pub interface_regex: SerializableRegex,
  pub graphs: NetworkGraphContainerProps,
}

#[derive(Deserialize)]
pub struct ConfigProps {
  pub width: u32,
  pub weather: WeatherProps,
  pub cpu_memory: CpuMemoryProps,
  pub disk: DiskProps,
  pub gpu: GpuProps,
  pub network: NetworkProps,
}

pub fn load_config() -> Result<ConfigProps, Box<dyn Error>> {
  let config_path = get_xdg_dirs().place_config_file("dogky.yaml")?;
  let config_file = File::open(config_path)?;
  let config_props: ConfigProps = serde_yml::from_reader(config_file)?;
  Ok(config_props)
}
