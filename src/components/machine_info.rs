use freya::prelude::*;
use freya::text_edit::Clipboard;
use sysinfo::System;

use crate::freya_utils::{center_cont, color_label, cursor_area};
use crate::styles_config::{GlobalStyles, MachineInfoStyles};

pub fn machine_info_component() -> Rect {
  let styles = use_consume::<MachineInfoStyles>();
  let global_styles = use_consume::<GlobalStyles>();

  let kernel_version = System::kernel_version().unwrap();
  let uname_info = uname::uname().unwrap();

  let center_cont = center_cont(global_styles.h_gap);

  center_cont.children([
    rect()
      .direction(Direction::Horizontal)
      .children([
        color_label(*styles.user_color, whoami::username().unwrap()),
        color_label(*styles.at_color, "@"),
        color_label(*styles.host_color, whoami::hostname().unwrap()),
      ])
      .into_element(),
    color_label(*styles.distro_color, whoami::distro().unwrap()).into_element(),
    cursor_area(CursorIcon::Copy)
      .child(
        color_label(*styles.kernel_version_color, kernel_version.clone())
          .on_pointer_press(move |_| Clipboard::set(kernel_version.clone()).unwrap()),
      )
      .into_element(),
    color_label(*styles.architecture_color, uname_info.machine).into_element(),
  ])
}
