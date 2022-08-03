use gtk::prelude::ObjectExt;
use sysinfo::{System, SystemExt};

use crate::components::MachineInfoWidget;

pub fn update_machine_info_widget(machine_info_widget: &MachineInfoWidget) {
  let system = System::new();
  let uname_info = uname::uname().unwrap();
  machine_info_widget.set_property("user", &whoami::username());
  machine_info_widget.set_property("host", &whoami::hostname());
  machine_info_widget.set_property("distro", &whoami::distro());
  machine_info_widget.set_property("kernel-version", &system.kernel_version());
  machine_info_widget.set_property("architecture", &uname_info.machine);
}
