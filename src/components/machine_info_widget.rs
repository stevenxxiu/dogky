use gtk::Builder;
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

  pub fn update(builder: &Builder) {
    let system = System::new();
    let uname_info = uname::uname().unwrap();
    set_label(builder, "user", &whoami::username());
    set_label(builder, "host", &whoami::hostname());
    set_label(builder, "distro", &whoami::distro());
    set_label(builder, "kernel_version", &system.kernel_version().unwrap());
    set_label(builder, "architecture", &uname_info.machine);
  }
}
