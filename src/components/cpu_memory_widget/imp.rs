use gtk::glib::subclass::InitializingObject;
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::prelude::InitializingWidgetExt;
use gtk::subclass::prelude::{
  BoxImpl, CompositeTemplateClass, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt, WidgetImpl,
};
use gtk::{glib, CompositeTemplate, Label, TemplateChild};
use once_cell::sync::Lazy;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/dogky/cpu_memory_widget.ui")]
pub struct CpuMemoryWidget {
  #[template_child]
  pub cpu_model: TemplateChild<Label>,
  #[template_child]
  pub cpu_frequency: TemplateChild<Label>,
  #[template_child]
  pub cpu_temperature: TemplateChild<Label>,
  #[template_child]
  pub cpu_usage: TemplateChild<Label>,
  #[template_child]
  pub system_uptime: TemplateChild<Label>,
  #[template_child]
  pub system_num_processes: TemplateChild<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for CpuMemoryWidget {
  // `NAME` needs to match `class` attribute of template
  const NAME: &'static str = "CpuMemoryWidget";
  type Type = super::CpuMemoryWidget;
  type ParentType = gtk::Box;

  fn class_init(klass: &mut Self::Class) {
    klass.bind_template();
  }

  fn instance_init(obj: &InitializingObject<Self>) {
    obj.init_template();
  }
}

impl ObjectImpl for CpuMemoryWidget {
  fn properties() -> &'static [ParamSpec] {
    static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
      vec![
        ParamSpecString::builder("cpu-model").build(),
        ParamSpecString::builder("cpu-frequency").build(),
        ParamSpecString::builder("cpu-temperature").build(),
        ParamSpecString::builder("cpu-usage").build(),
        ParamSpecString::builder("system-uptime").build(),
        ParamSpecString::builder("system-num-processes").build(),
      ]
    });
    PROPERTIES.as_ref()
  }

  fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "cpu-model" => self.cpu_model.set_label(value.get().unwrap()),
      "cpu-frequency" => self.cpu_frequency.set_label(value.get().unwrap()),
      "cpu-temperature" => self.cpu_temperature.set_label(value.get().unwrap()),
      "cpu-usage" => self.cpu_usage.set_label(value.get().unwrap()),
      "system-uptime" => self.system_uptime.set_label(value.get().unwrap()),
      "system-num-processes" => self.system_num_processes.set_label(value.get().unwrap()),
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for CpuMemoryWidget {}

impl BoxImpl for CpuMemoryWidget {}
