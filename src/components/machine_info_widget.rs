use gtk::prelude::{DisplayExt, GestureExt, WidgetExt};
use gtk::{glib, Builder, Label};
use sysinfo::{System, SystemExt};

use crate::gtk_utils::set_label;

pub struct MachineInfoWidget {}

impl MachineInfoWidget {
  pub fn build() -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/machine_info_widget.ui");
    let container: gtk::Box = builder.object("machine_info_widget").unwrap();
    MachineInfoWidget::update(&builder);
    container
  }

  fn update_kernel_version(builder: &Builder) {
    let system = System::new();
    let kernel_version_label = builder.object::<Label>("kernel_version").unwrap();
    let kernel_version = system.kernel_version().unwrap();
    set_label(builder, "kernel_version", &kernel_version);

    let gesture = gtk::GestureClick::new();
    gesture.connect_released(glib::clone!(@strong kernel_version_label => move |gesture, _, _, _| {
      gesture.set_state(gtk::EventSequenceState::Claimed);
      kernel_version_label.display().clipboard().set_text(&kernel_version);
    }));
    kernel_version_label.add_controller(&gesture);
    kernel_version_label.set_cursor_from_name(Option::from("copy"));
  }

  pub fn update(builder: &Builder) {
    let uname_info = uname::uname().unwrap();
    set_label(builder, "user", &whoami::username());
    set_label(builder, "host", &whoami::hostname());
    set_label(builder, "distro", &whoami::distro());
    MachineInfoWidget::update_kernel_version(builder);
    set_label(builder, "architecture", &uname_info.machine);
  }
}
