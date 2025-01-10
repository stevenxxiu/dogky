extern crate xdg;

use std::error::Error;
use std::fs::File;

use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::{SerdeCommand, SerdeRegex};

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
}

#[derive(Deserialize)]
pub struct CpuMemoryProcessListProps {
  pub num_processes: usize,
  pub pid_width: f32,
  pub cpu_width: f32,
  pub memory_width: f32,
  pub top_command: SerdeCommand,
}

#[derive(Deserialize)]
pub struct CpuMemoryProps {
  pub update_interval: u64,
  pub cpu_bars: CpuBarsProps,
  pub process_list: CpuMemoryProcessListProps,
}

#[derive(Deserialize)]
pub struct DiskProps {
  pub update_interval: u64,
  pub name: String,
  pub device_path: String,
  pub mount_point: String,
}

#[derive(Deserialize)]
pub struct GpuProps {
  pub update_interval: u64,
}

#[derive(Deserialize)]
pub struct NetworkGraphProps {
  pub maximum_bytes_per_sec: u64,
}

#[derive(Deserialize)]
pub struct NetworkGraphContainerProps {
  pub upload: NetworkGraphProps,
  pub download: NetworkGraphProps,
}

#[derive(Deserialize)]
pub struct NetworkProps {
  pub update_interval: u64,
  pub public_ip_retry_timeout: Option<u64>,
  pub interface_regex: SerdeRegex,
  pub graphs: NetworkGraphContainerProps,
}

#[derive(Deserialize)]
pub struct ConfigProps {
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
