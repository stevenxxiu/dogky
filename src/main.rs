use std::sync::Arc;

use freya::prelude::*;
use nvml_wrapper::Nvml;
use styles_config::{GlobalStyles, StylesConfig};
use velcro::vec;
use winit::platform::wayland::WindowAttributesExtWayland as _;
use winit::window::WindowLevel;

use components::{
  cpu_memory_component, disk_component, machine_info_component, network_component, weather_component, GpuComponent,
};
use custom_components::create_separator;

mod api;
mod components;
mod config;
mod custom_components;
mod format_size;
mod freya_utils;
mod move_window;
mod path;
mod serde_structs;
mod styles_config;
mod utils;

struct App {
  styles: StylesConfig,
}

impl Render for App {
  fn render(&self) -> impl IntoElement {
    let global_styles = GlobalStyles {
      container_width: self.styles.width as f32 - self.styles.padding.left() - self.styles.padding.right(),
      h_gap: self.styles.h_gap,
    };
    provide_context(global_styles);
    provide_context(self.styles.weather.clone());
    provide_context(self.styles.machine_info.clone());
    provide_context(self.styles.cpu_memory.clone());
    provide_context(self.styles.disk.clone());
    provide_context(self.styles.gpu.clone());
    provide_context(self.styles.network.clone());

    let config = config::load_config().unwrap();
    provide_context(config.weather);
    provide_context(config.cpu_memory);
    provide_context(config.disk);
    provide_context(config.gpu);
    provide_context(config.network);

    let separator = create_separator(self.styles.separator_height);

    rect()
      .width(Size::percent(100.))
      .height(Size::percent(100.))
      .background(*self.styles.background_color)
      .color(*self.styles.text_color)
      .font_size(self.styles.text_size)
      .padding(*self.styles.padding)
      .children(vec![
        weather_component().into(),
        separator.clone().into(),
        machine_info_component().into(),
        separator.clone().into(),
        cpu_memory_component().into(),
        separator.clone().into(),
        disk_component().into(),
        ..Nvml::init().map_or(vec![], |nvml| {
          vec![separator.clone().into(), GpuComponent { nvml: Arc::from(nvml) }.into()]
        }),
        separator.clone().into(),
        network_component(),
      ])
  }
}

fn main() {
  let styles = styles_config::load_config().unwrap();
  let font = styles.font.clone();
  let width = styles.width;

  // Build a *Tokio* runtime manually, to not interfere with *Freya*. Otherwise it hangs eventually.
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_time()
    .enable_io()
    .build()
    .unwrap();
  let _rt = rt.enter();

  launch(
    LaunchConfig::new().with_default_font(font).with_window(
      WindowConfig::new(FpRender::from_render(App { styles }))
        .with_window_handle(move |_window| move_window::move_window(width).unwrap())
        .with_background(Color::TRANSPARENT)
        .with_window_attributes(|attributes, _| {
          attributes
            .with_name("dogky", "")
            .with_title("Dogky")
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnBottom)
        }),
    ),
  );
}
