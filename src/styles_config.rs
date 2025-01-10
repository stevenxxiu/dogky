extern crate xdg;

use std::error::Error;
use std::fs::File;

use iced::Size;
use serde_derive::Deserialize;

use crate::path::get_xdg_dirs;
use crate::serde_structs::{SerdeColor, SerdePadding};

#[derive(Deserialize, Clone)]
pub struct WeatherStyles {
  pub container_padding: SerdePadding,
  pub cond_icon_size: f32,
  pub cond_icon_padding: SerdePadding,
  pub value_color: SerdeColor,
  pub wind_arrow_offset: f32,
  pub wind_arrow_size: f32,
  pub wind_arrow_canvas_size: f32,
  pub sunrise_icon_padding: SerdePadding,
  pub sunset_icon_padding: SerdePadding,
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
pub struct CPUMemoryStyles {
  pub value_color: SerdeColor,

  pub bars_v_gap: f32,
  pub bar_height: f32,
  pub bar_border_color: SerdeColor,
  pub bar_fill_color: SerdeColor,
  pub bar_h_gap: f32,

  pub graph_h_gap: f32,
  pub graph_height: f32,
  pub graph_cpu_border_color: SerdeColor,
  pub graph_cpu_fill_color: SerdeColor,
  pub graph_memory_border_color: SerdeColor,
  pub graph_memory_fill_color: SerdeColor,
  pub graph_swap_fill_color: SerdeColor,

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
  pub bar_fill_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct GPUStyles {
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
  pub graph_download_fill_color: SerdeColor,
  pub graph_upload_border_color: SerdeColor,
  pub graph_upload_fill_color: SerdeColor,
}

#[derive(Deserialize, Clone)]
pub struct StylesConfig {
  pub antialiasing: bool,
  pub width: u32,
  pub background_color: SerdeColor,
  pub text_size: f32,
  pub text_color: SerdeColor,
  pub padding: SerdePadding,
  pub separator_padding: SerdePadding,
  pub h_gap: f32,
  pub border_width: f32,

  pub weather: WeatherStyles,
  pub machine_info: MachineInfoStyles,
  pub cpu_memory: CPUMemoryStyles,
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
  pub h_gap: f32,
  pub border_width: f32,
  pub char_dims: Size,
}
