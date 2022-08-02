use std::process::Command;
use std::rc::Rc;

use gtk::gdk::Display;
use gtk::glib::clone;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt};
use gtk::{gio, Application, CssProvider, StyleContext};

use crate::components::Window;
use crate::config::{Config, ConfigProps};

mod api;
mod components;
mod config;
mod path;

const APP_ID: &str = "org.dogky";

fn load_css(css_bytes: &Option<Vec<u8>>) {
  let display = Display::default().expect("Could not connect to a display.");
  let priority = gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;

  let provider_static = CssProvider::new();
  provider_static.load_from_data(include_bytes!("resources/style.css"));
  StyleContext::add_provider_for_display(&display, &provider_static, priority);

  if let Some(css_bytes) = css_bytes {
    let provider_custom = CssProvider::new();
    provider_custom.load_from_data(css_bytes);
    StyleContext::add_provider_for_display(&display, &provider_custom, priority);
  }
}

fn move_window(app: &Application, window: &Window, config_props: &ConfigProps) {
  // Set initial opacity to 0, to avoid flickering when `move_window.sh` runs
  window.set_opacity(0f64);
  let (monitor_width, monitor_height, window_width, window_height) = config_props.calculate_size(window);
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

fn build_ui(app: &Application, config_props: &ConfigProps) {
  let window = Window::new(app, config_props);
  window.present();
  move_window(app, &window, config_props);
}

fn main() {
  let config = Config::load().unwrap();
  let config_ref = Rc::new(config);
  gio::resources_register_include!("dogky.gresource").unwrap();
  let app = Application::builder().application_id(APP_ID).build();
  app.connect_startup(clone!(@strong config_ref => move |_| load_css(&config_ref.css_bytes)));
  app.connect_activate(clone!(@strong config_ref => move |app| build_ui(app, &config_ref.config_props)));
  app.run();
}
