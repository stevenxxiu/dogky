use std::process::Command;

use gtk4::gdk::Display;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt};
use gtk4::{Application, ApplicationWindow, Box, CssProvider, Orientation, StyleContext};

mod config;
mod serializable_rgba;

use crate::config::Config;

const APP_ID: &str = "org.dogky";

fn load_css(config: &Config) {
  let provider_static = CssProvider::new();
  provider_static.load_from_data(include_bytes!("style.css"));

  let provider_background = CssProvider::new();
  provider_background.load_from_data(format!("window {{ background-color: {}; }}", config.bg_color).as_bytes());

  for provider in [&provider_static, &provider_background] {
    StyleContext::add_provider_for_display(
      &Display::default().expect("Could not connect to a display."),
      provider,
      gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
  }
}

fn build_ui(app: &Application, config: &Config) {
  let container = Box::new(Orientation::Vertical, config.margin as i32);
  let window = ApplicationWindow::builder()
    .application(app)
    .child(&container)
    .decorated(false)
    .build();
  window.present();

  // Set initial opacity to 0, to avoid flickering when `move_window.sh` runs
  window.set_opacity(0f64);
  let (monitor_width, monitor_height, window_width, window_height) = config.calculate_size(&window);
  window.set_size_request(window_width as i32, window_height as i32);

  let move_window_script_path = std::env::current_exe()
    .unwrap()
    .parent()
    .unwrap()
    .join("move_window.sh");
  let output = Command::new(move_window_script_path)
    .args([monitor_width, monitor_height, window_width, window_height].map(|n| n.to_string()))
    .output()
    .unwrap();
  if !output.status.success() {
    app.quit();
    return;
  }
  window.set_opacity(1f64);
}

fn main() {
  let config = Config::load().unwrap();
  let app = Application::builder().application_id(APP_ID).build();
  app.connect_startup(move |_| load_css(&config));
  app.connect_activate(move |app| build_ui(app, &config));
  app.run();
}
