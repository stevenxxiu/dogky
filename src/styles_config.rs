extern crate xdg;

use std::error::Error;
use std::fs::File;

use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::SerdeColor;
use crate::serde_structs::SerdeGaps;

#[derive(Deserialize, Clone)]
pub struct WeatherStyles {
  pub container_padding: SerdeGaps,
  pub cond_icon_size: f32,
  pub value_color: SerdeColor,
  pub wind_arrow_margin: SerdeGaps,
}

#[derive(Deserialize, Clone)]
pub struct MachineInfoStyles {
  pub user_color: SerdeColor,
  pub at_color: SerdeColor,
  pub host_color: SerdeColor,
  pub distro_color: SerdeColor,
  pub architecture_color: SerdeColor,
  pub kernel_version_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct CpuMemoryStyles {
  pub value_color: SerdeColor,

  pub bars_per_row: usize,
  pub bars_v_gap: f32,
  pub bar_height: f32,
  pub bar_border_color: SerdeColor,
  pub bar_border_width: f32,
  pub bar_fill_color: SerdeColor,
  pub bar_efficient_border_color: SerdeColor,
  pub bar_efficient_border_width: f32,
  pub bar_efficient_fill_color: SerdeColor,
  pub bar_h_gap: f32,

  pub graph_h_gap: f32,
  pub graph_height: f32,
  pub graph_cpu_border_color: SerdeColor,
  pub graph_cpu_border_width: f32,
  pub graph_cpu_fill_color: SerdeColor,
  pub graph_memory_border_color: SerdeColor,
  pub graph_memory_border_width: f32,
  pub graph_memory_fill_color: SerdeColor,
  pub graph_swap_fill_color: SerdeColor,

  pub ps_pid_width: f32,
  pub ps_cpu_width: f32,
  pub ps_memory_width: f32,
  pub ps_header_color: SerdeColor,
  pub ps_sort_cpu_color: SerdeColor,
  pub ps_sort_memory_color: SerdeColor,
  pub ps_cpu_color: SerdeColor,
  pub ps_memory_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct DiskStyles {
  pub name_color: SerdeColor,
  pub value_color: SerdeColor,
  pub bar_height: f32,
  pub bar_border_color: SerdeColor,
  pub bar_border_width: f32,
  pub bar_fill_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct GpuStyles {
  pub name_color: SerdeColor,
  pub usage_name_color: SerdeColor,
  pub value_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct NetworkStyles {
  pub name_color: SerdeColor,
  pub value_color: SerdeColor,

  pub graph_h_gap: f32,
  pub graph_height: f32,
  pub graph_download_border_color: SerdeColor,
  pub graph_download_border_width: f32,
  pub graph_download_fill_color: SerdeColor,
  pub graph_upload_border_color: SerdeColor,
  pub graph_upload_border_width: f32,
  pub graph_upload_fill_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct StylesConfig {
  pub width: u32,
  pub background_color: SerdeColor,
  pub font: String,
  pub text_size: f32,
  pub text_color: SerdeColor,
  pub padding: SerdeGaps,
  pub separator_height: f32,
  pub h_gap: f32,

  pub weather: WeatherStyles,
  pub machine_info: MachineInfoStyles,
  pub cpu_memory: CpuMemoryStyles,
  pub disk: DiskStyles,
  pub gpu: GpuStyles,
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
  pub h_gap: f32,
}
