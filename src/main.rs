use std::process::Command;
use std::rc::Rc;

use gtk::gdk::Display;
use gtk::glib::clone;
use gtk::prelude::{ApplicationExt, ApplicationExtManual, GtkWindowExt, WidgetExt};
use gtk::{Application, ApplicationWindow, Box, CssProvider, Orientation, StyleContext};

mod api;
mod components;
mod config;
mod path;

use crate::components::WeatherWidget;
use crate::config::{Config, ConfigProps};

const APP_ID: &str = "org.dogky";

fn load_css(css_bytes: &Vec<u8>) {
  let provider_styles = CssProvider::new();
  provider_styles.load_from_data(&css_bytes);
  StyleContext::add_provider_for_display(
    &Display::default().expect("Could not connect to a display."),
    &provider_styles,
    gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
  );
}

fn create_container(config_props: &ConfigProps) -> Box {
  let container = Box::new(Orientation::Vertical, config_props.margin as i32);
  WeatherWidget::create(&config_props.weather, &container);

  container
}

fn move_window(app: &Application, window: &ApplicationWindow, config_props: &ConfigProps) {
  // Set initial opacity to 0, to avoid flickering when `move_window.sh` runs
  window.set_opacity(0f64);
  let (monitor_width, monitor_height, window_width, window_height) = config_props.calculate_size(&window);
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

fn create_window(app: &Application, container: &Box, config_props: &ConfigProps) {
  let window = ApplicationWindow::builder()
    .application(app)
    .child(container)
    .decorated(false)
    .build();
  window.present();
  move_window(app, &window, config_props);
}

fn build_ui(app: &Application, config_props: &ConfigProps) {
  let container = create_container(config_props);
  create_window(app, &container, config_props);
}

fn main() {
  let config = Config::load().unwrap();
  let config_ref = Rc::new(config);
  let app = Application::builder().application_id(APP_ID).build();
  app.connect_startup(clone!(@strong config_ref => move |_| load_css(&config_ref.css_bytes)));
  app.connect_activate(clone!(@strong config_ref => move |app| build_ui(app, &config_ref.config_props)));
  app.run();
}
