use std::process::Command;

use freya::prelude::*;
use freya_core::parsing::Parse;
use nvml_wrapper::Nvml;
use styles_config::{GlobalStyles, StylesConfig};
use winit::platform::wayland::WindowAttributesExtWayland as _;
use winit::window::WindowLevel;

use components::{
  CpuMemoryComponent, DiskComponent, GpuComponent, MachineInfoComponent, NetworkComponent, WeatherComponent,
};
use custom_components::Separator;

mod api;
mod components;
mod config;
mod custom_components;
mod format_size;
mod path;
mod serde_structs;
mod styles_config;
mod utils;

fn app() -> Element {
  let styles = consume_context::<StylesConfig>();
  let padding_parsed = Gaps::parse(&styles.padding).unwrap();
  let global_styles = GlobalStyles {
    container_width: styles.width as f32 - padding_parsed.left() - padding_parsed.right(),
    h_gap: styles.h_gap,
  };
  use_context_provider(|| global_styles);
  use_context_provider(|| styles.weather);
  use_context_provider(|| styles.machine_info);
  use_context_provider(|| styles.cpu_memory);
  use_context_provider(|| styles.disk);
  use_context_provider(|| styles.gpu);
  use_context_provider(|| styles.network);

  let config = config::load_config().unwrap();
  use_context_provider(|| config.weather);
  use_context_provider(|| config.cpu_memory);
  use_context_provider(|| config.disk);
  use_context_provider(|| config.gpu);
  use_context_provider(|| config.network);

  let nvml_res = Nvml::init();

  rsx!(rect {
    width: "100%",
    height: "100%",
    direction: "vertical",
    background: styles.background_color,
    color: styles.text_color,
    font_size: styles.text_size.to_string(),
    padding: styles.padding.clone(),
    WeatherComponent {}
    Separator { height: styles.separator_height.clone() }
    MachineInfoComponent {}
    Separator { height: styles.separator_height.clone() }
    CpuMemoryComponent {}
    Separator { height: styles.separator_height.clone() }
    DiskComponent {}
    if let Ok(nvml) = nvml_res {
      Separator { height: styles.separator_height.clone() }
      GpuComponent { nvml_signal: nvml }
    }
    Separator { height: styles.separator_height.clone() }
    NetworkComponent {}
  })
}

pub fn main() {
  let styles = styles_config::load_config().unwrap();
  let font = styles.font.clone();
  let width = styles.width;
  launch_cfg(
    app,
    LaunchConfig::<StylesConfig> {
      state: Some(styles),
      default_fonts: vec![font],
      ..Default::default()
    }
    .on_setup(move |_window| {
      let move_window_script_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("move_window.py");
      Command::new(move_window_script_path)
        .arg(width.to_string())
        .output()
        .unwrap();
    })
    .with_background("transparent")
    .with_window_attributes(|attributes| {
      attributes
        .with_name("dogky", "")
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnBottom)
    }),
  )
}
