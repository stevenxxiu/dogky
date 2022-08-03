use gtk::glib::subclass::InitializingObject;
use gtk::glib::{ParamSpec, ParamSpecString, Value};
use gtk::prelude::InitializingWidgetExt;
use gtk::subclass::prelude::{
  BoxImpl, CompositeTemplateClass, ObjectImpl, ObjectSubclass, WidgetClassSubclassExt, WidgetImpl,
};
use gtk::{glib, Box, CompositeTemplate, Label, TemplateChild};
use once_cell::sync::Lazy;

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/dogky/machine_info_widget.ui")]
pub struct MachineInfoWidget {
  #[template_child]
  pub machine_info_container: TemplateChild<Box>,
  #[template_child]
  pub user: TemplateChild<Label>,
  #[template_child]
  pub host: TemplateChild<Label>,
  #[template_child]
  pub distro: TemplateChild<Label>,
  #[template_child]
  pub kernel_version: TemplateChild<Label>,
  #[template_child]
  pub architecture: TemplateChild<Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for MachineInfoWidget {
  // `NAME` needs to match `class` attribute of template
  const NAME: &'static str = "MachineInfoWidget";
  type Type = super::MachineInfoWidget;
  type ParentType = gtk::Box;

  fn class_init(klass: &mut Self::Class) {
    klass.bind_template();
  }

  fn instance_init(obj: &InitializingObject<Self>) {
    obj.init_template();
  }
}

impl ObjectImpl for MachineInfoWidget {
  fn properties() -> &'static [ParamSpec] {
    static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
      vec![
        ParamSpecString::builder("user").build(),
        ParamSpecString::builder("host").build(),
        ParamSpecString::builder("distro").build(),
        ParamSpecString::builder("kernel-version").build(),
        ParamSpecString::builder("architecture").build(),
      ]
    });
    PROPERTIES.as_ref()
  }

  fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
    match pspec.name() {
      "user" => self.user.set_label(value.get().unwrap()),
      "host" => self.host.set_label(value.get().unwrap()),
      "distro" => self.distro.set_label(value.get().unwrap()),
      "kernel-version" => self.kernel_version.set_label(value.get().unwrap()),
      "architecture" => self.architecture.set_label(value.get().unwrap()),
      _ => unimplemented!(),
    }
  }
}

impl WidgetImpl for MachineInfoWidget {}

impl BoxImpl for MachineInfoWidget {}
