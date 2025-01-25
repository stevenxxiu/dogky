extern crate xdg;

use std::error::Error;
use std::fs::File;

use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::SerdeColor;

#[derive(Deserialize, Clone)]
pub struct WeatherStyles {
  pub container_padding: String,
  pub cond_icon_size: f32,
  pub value_color: String,
  pub wind_arrow_margin: String,
}

#[derive(Deserialize, Clone)]
pub struct MachineInfoStyles {
  pub user_color: String,
  pub at_color: String,
  pub host_color: String,
  pub distro_color: String,
  pub architecture_color: String,
  pub kernel_version_color: String,
}

#[derive(Deserialize, Clone)]
pub struct CpuMemoryStyles {
  pub value_color: String,

  pub bars_per_row: usize,
  pub bars_v_gap: f32,
  pub bar_height: f32,
  pub bar_border: String,
  pub bar_fill_color: String,
  pub bar_h_gap: f32,

  pub graph_h_gap: f32,
  pub graph_height: u32,
  pub graph_cpu_border: String,
  pub graph_cpu_fill_color: SerdeColor,
  pub graph_memory_border: String,
  pub graph_memory_fill_color: SerdeColor,
  pub graph_swap_fill_color: SerdeColor,

  pub ps_pid_width: f32,
  pub ps_cpu_width: f32,
  pub ps_memory_width: f32,
  pub ps_header_color: String,
  pub ps_sort_cpu_color: String,
  pub ps_sort_memory_color: String,
  pub ps_cpu_color: String,
  pub ps_memory_color: String,
}

#[derive(Deserialize, Clone)]
pub struct DiskStyles {
  pub name_color: String,
  pub value_color: String,
  pub bar_height: f32,
  pub bar_border_color: String,
  pub bar_fill_color: String,
}

#[derive(Deserialize, Clone)]
pub struct GPUStyles {
  pub name_color: String,
  pub usage_name_color: String,
  pub value_color: String,
}

#[derive(Deserialize, Clone)]
pub struct NetworkStyles {
  pub name_color: String,
  pub value_color: String,

  pub graph_h_gap: f32,
  pub graph_height: f32,
  pub graph_download_border_color: String,
  pub graph_download_fill_color: String,
  pub graph_upload_border_color: String,
  pub graph_upload_fill_color: String,
}

#[derive(Deserialize, Clone)]
pub struct StylesConfig {
  pub width: u32,
  pub background_color: String,
  pub font: String,
  pub text_size: f32,
  pub text_color: String,
  pub padding: String,
  pub separator_padding: String,
  pub h_gap: f32,

  pub weather: WeatherStyles,
  pub machine_info: MachineInfoStyles,
  pub cpu_memory: CpuMemoryStyles,
  pub disk: DiskStyles,
  pub gpu: GPUStyles,
  pub network: NetworkStyles,
}

pub fn load_config() -> Result<StylesConfig, Box<dyn Error>> {
  let config_path = get_xdg_dirs().place_config_file("styles.yaml")?;
  let config_file = File::open(config_path)?;
  let config_styles: StylesConfig = serde_yml::from_reader(config_file)?;
  Ok(config_styles)
}

#[derive(Clone)]
pub struct GlobalStyles {
  pub container_width: f32,
  pub padding: String,
  pub h_gap: f32,
}
