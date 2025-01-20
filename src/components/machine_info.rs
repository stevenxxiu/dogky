use copypasta::{ClipboardContext, ClipboardProvider};
use freya::prelude::*;
use sysinfo::System;

use crate::styles_config::{GlobalStyles, MachineInfoStyles};

#[allow(non_snake_case)]
#[component]
pub fn MachineInfoComponent(styles: MachineInfoStyles) -> Element {
  let global_styles = use_context::<GlobalStyles>();
  let mut ctx = ClipboardContext::new().unwrap();

  let kernel_version = System::kernel_version().unwrap();
  let uname_info = uname::uname().unwrap();

  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      spacing: global_styles.h_gap.to_string(),
      main_align: "center",
      rect {
        direction: "horizontal",
        label { color: styles.user_color, "{whoami::username()}" }
        label { color: styles.at_color, "@" }
        label { color: styles.host_color, "{whoami::fallible::hostname().unwrap()}" }
      }
      label { color: styles.distro_color, "{whoami::distro()}" }
      CursorArea {
        icon: CursorIcon::Copy,
        label {
          color: styles.kernel_version_color,
          onclick: move |_| { let _ = ctx.set_contents(kernel_version.clone()); },
          "{kernel_version}"
        }
      }
      label { color: styles.architecture_color, "{uname_info.machine}" }
    }
  )
}
