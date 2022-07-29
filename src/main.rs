use gtk4::gdk::Display;
use gtk4::gio::{Cancellable, SimpleAction};
use gtk4::glib;
use gtk4::glib::clone;
use gtk4::prelude::{
  ActionGroupExt, ActionMapExt, ApplicationExt, ApplicationExtManual, GtkApplicationExt, GtkWindowExt, WidgetExt,
};
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
  if app.active_window().is_some() {
    // This function can run again, when the move window script runs our binary again
    return;
  }
  let container = Box::new(Orientation::Vertical, config.margin as i32);
  let window = ApplicationWindow::builder()
    .application(app)
    .child(&container)
    .decorated(false)
    .build();
  window.present();

  // Set initial opacity to 0, to avoid flickering when the script that positions the window runs
  window.set_opacity(0f64);
  let action_position_finished = SimpleAction::new("make-visible", None);
  action_position_finished.connect_activate(clone!(@weak window => move |_, _| {
    window.set_opacity(1f64);
  }));
  app.add_action(&action_position_finished);

  let (width, height) = config.calculate_size(&window);
  window.set_size_request(width as i32, height as i32);
}

fn main() {
  let config = Config::load().unwrap();
  let app = Application::builder().application_id(APP_ID).build();
  app.register(Cancellable::NONE).unwrap();
  if app.is_remote() {
    app.activate_action("make-visible", None);
    app.connect_activate(move |app| app.quit());
  } else {
    app.connect_startup(move |_| load_css(&config));
    app.connect_activate(move |app| build_ui(app, &config));
  }
  app.run();
}
