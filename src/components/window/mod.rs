use glib::Object;
use gtk::{gio, glib, Application};

use crate::ConfigProps;

mod imp;

glib::wrapper! {
  pub struct Window(ObjectSubclass<imp::Window>)
    @extends gtk::Window, gtk::Widget,
    @implements
      gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native,
      gtk::Root, gtk::ShortcutManager;
}

impl Window {
  pub fn new(app: &Application, config_props: &ConfigProps) -> Self {
    let config_props_str = serde_json::to_string(config_props).unwrap();
    Object::new(&[("application", app), ("config-props-str", &config_props_str)]).unwrap()
  }
}
