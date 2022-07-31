use gtk::glib::subclass::InitializingObject;
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::prelude::{InitializingWidgetExt, WidgetExt};
use gtk::subclass::prelude::{
  BoxImpl, CompositeTemplateClass, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt, WidgetImpl,
};
use gtk::{glib, Box, CompositeTemplate, Label, TemplateChild};
use once_cell::sync::Lazy;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/dogky/weather_widget.ui")]
pub struct WeatherWidget {
  #[template_child]
  pub error_container: TemplateChild<Box>,
  #[template_child]
  pub error: TemplateChild<Label>,
  #[template_child]
  pub icon: TemplateChild<Label>,
  #[template_child]
  pub conditions: TemplateChild<Label>,
  #[template_child]
  pub temperature: TemplateChild<Label>,
  #[template_child]
  pub humidity: TemplateChild<Label>,
  #[template_child]
  pub wind: TemplateChild<Label>,
  #[template_child]
  pub sunset: TemplateChild<Label>,
  #[template_child]
  pub sunrise: TemplateChild<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for WeatherWidget {
  // `NAME` needs to match `class` attribute of template
  const NAME: &'static str = "WeatherWidget";
  type Type = super::WeatherWidget;
  type ParentType = gtk::Box;

  fn class_init(klass: &mut Self::Class) {
    klass.bind_template();
  }

  fn instance_init(obj: &InitializingObject<Self>) {
    obj.init_template();
  }
}

impl ObjectImpl for WeatherWidget {
  fn properties() -> &'static [ParamSpec] {
    static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
      vec![
        ParamSpecString::builder("error").build(),
        ParamSpecString::builder("icon").build(),
        ParamSpecString::builder("conditions").build(),
        ParamSpecString::builder("temperature").build(),
        ParamSpecString::builder("humidity").build(),
        ParamSpecString::builder("wind").build(),
        ParamSpecString::builder("sunrise").build(),
        ParamSpecString::builder("sunset").build(),
      ]
    });
    PROPERTIES.as_ref()
  }

  fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "error" => match value.get::<Option<String>>().unwrap() {
        Some(error_text) => {
          self.error_container.set_visible(true);
          self.error.set_label(&error_text);
        }
        None => self.error_container.set_visible(false),
      },
      "icon" => self.icon.set_label(value.get().unwrap()),
      "conditions" => self.conditions.set_label(value.get().unwrap()),
      "temperature" => self.temperature.set_label(value.get().unwrap()),
      "humidity" => self.humidity.set_label(value.get().unwrap()),
      "wind" => self.wind.set_label(value.get().unwrap()),
      "sunrise" => self.sunrise.set_label(value.get().unwrap()),
      "sunset" => self.sunset.set_label(value.get().unwrap()),
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for WeatherWidget {}

impl BoxImpl for WeatherWidget {}
