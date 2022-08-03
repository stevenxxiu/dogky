use gtk::glib;
use gtk::glib::subclass::InitializingObject;
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::prelude::InitializingWidgetExt;
use gtk::subclass::prelude::{
  CompositeTemplateClass, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt, WidgetImpl, WindowImpl,
};
use gtk::CompositeTemplate;
use gtk::TemplateChild;
use once_cell::sync::Lazy;

use crate::components::{update_machine_info_widget, MachineInfoWidget, WeatherWidget, WeatherWidgetUpdater};
use crate::ConfigProps;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/dogky/window.ui")]
pub struct Window {
  #[template_child]
  pub weather_widget: TemplateChild<WeatherWidget>,
  #[template_child]
  pub machine_info_widget: TemplateChild<MachineInfoWidget>,
}

#[glib::object_subclass]
impl ObjectSubclass for Window {
  // `NAME` needs to match `class` attribute of template
  const NAME: &'static str = "DogkyWindow";
  type Type = super::Window;
  type ParentType = gtk::Window;

  fn class_init(klass: &mut Self::Class) {
    klass.bind_template();
  }

  fn instance_init(obj: &InitializingObject<Self>) {
    obj.init_template();
  }
}

impl ObjectImpl for Window {
  fn properties() -> &'static [ParamSpec] {
    static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| vec![ParamSpecString::builder("config-props-str").build()]);
    PROPERTIES.as_ref()
  }

  fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "config-props-str" => {
        let config_props: ConfigProps = serde_json::from_str(value.get().unwrap()).unwrap();
        WeatherWidgetUpdater::init(config_props.weather, &self.weather_widget);
        update_machine_info_widget(&self.machine_info_widget);
      }
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}
