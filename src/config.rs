extern crate xdg;

use std::error::Error;
use std::fs::File;

use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::{SerdeCommand, SerdeRegex};

#[derive(Deserialize, Clone)]
pub struct WeatherConfig {
  pub update_interval: u64,
  pub retry_timeout: u64,
  pub openweather_api_key: String,
  pub openweather_city_id: u64,
}

#[derive(Deserialize, Clone)]
pub struct CpuMemoryProcessListConfig {
  pub num_processes: usize,
  pub top_command: SerdeCommand,
}

#[derive(Deserialize, Clone)]
pub struct CpuMemoryConfig {
  pub update_interval: u64,
  pub process_list: CpuMemoryProcessListConfig,
}

#[derive(Deserialize, Clone)]
pub struct DiskConfig {
  pub update_interval: u64,
  pub name: String,
  pub device_path: String,
  pub mount_point: String,
}

#[derive(Deserialize, Clone)]
pub struct GpuConfig {
  pub update_interval: u64,
}

#[derive(Deserialize, Clone)]
pub struct NetworkGraphProps {
  pub maximum_bytes_per_sec: u64,
}

#[derive(Deserialize, Clone)]
pub struct NetworkGraphContainerProps {
  pub upload: NetworkGraphProps,
  pub download: NetworkGraphProps,
}

#[derive(Deserialize, Clone)]
pub struct NetworkConfig {
  pub update_interval: u64,
  pub public_ip_retry_timeout: Option<u64>,
  pub interface_regex: SerdeRegex,
  pub graphs: NetworkGraphContainerProps,
}

#[derive(Deserialize, Clone)]
pub struct DogkyConfig {
  pub weather: WeatherConfig,
  pub cpu_memory: CpuMemoryConfig,
  pub disk: DiskConfig,
  pub gpu: GpuConfig,
  pub network: NetworkConfig,
}

pub fn load_config() -> Result<DogkyConfig, Box<dyn Error>> {
  let config_path = get_xdg_dirs().place_config_file("dogky.yaml")?;
  let config_file = File::open(config_path)?;
  let config: DogkyConfig = serde_yml::from_reader(config_file)?;
  Ok(config)
}
