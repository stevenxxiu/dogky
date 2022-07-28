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
  let container = Box::new(Orientation::Vertical, 0);
  let window = ApplicationWindow::builder().application(app).child(&container).decorated(false).build();
  window.present();
  let (width, height) = config.calculate_size(&window);
  window.set_size_request(width as i32, height as i32);
}

fn main() {
  let config = Config::load().unwrap();
  let app = Application::builder().application_id(APP_ID).build();

  app.connect_startup(move |_| load_css(&config));
  app.connect_activate(move |app| build_ui(app, &config));
  app.run();
}
