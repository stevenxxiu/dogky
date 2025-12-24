use freya::prelude::*;
use freya::text_edit::Clipboard;
use sysinfo::System;

use crate::freya_utils::{center_cont_factory, color_label, cursor_area};
use crate::styles_config::{GlobalStyles, MachineInfoStyles};

pub fn machine_info_component() -> Rect {
  let styles = use_consume::<MachineInfoStyles>();
  let global_styles = use_consume::<GlobalStyles>();

  let kernel_version = System::kernel_version().unwrap();
  let uname_info = uname::uname().unwrap();

  let center_cont = center_cont_factory(global_styles.h_gap);

  center_cont([
    rect()
      .direction(Direction::Horizontal)
      .children([
        color_label(*styles.user_color, whoami::username()).into(),
        color_label(*styles.at_color, "@").into(),
        color_label(*styles.host_color, whoami::fallible::hostname().unwrap()).into(),
      ])
      .into(),
    color_label(*styles.distro_color, whoami::distro()).into(),
    cursor_area(CursorIcon::Copy)
      .child(
        color_label(*styles.kernel_version_color, kernel_version.clone())
          .on_pointer_press(move |_| Clipboard::set(kernel_version.clone()).unwrap()),
      )
      .into(),
    color_label(*styles.architecture_color, uname_info.machine).into(),
  ])
}
