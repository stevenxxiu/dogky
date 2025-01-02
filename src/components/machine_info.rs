use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{column, container, mouse_area, row, text};
use iced::{clipboard, Task};
use iced::{Element, Length};
use sysinfo::{System, SystemExt};

use crate::message::Message;
use crate::styles::machine_info as styles;
use crate::ui_utils::space_row;

#[derive(Default)]
pub struct MachineInfoComponent {
  username: String,
  hostname: String,
  distro: String,
  kernel_version: String,
  architecture: String,
}

impl MachineInfoComponent {
  pub fn new() -> Self {
    let system = System::new();
    let uname_info = uname::uname().unwrap();
    Self {
      username: whoami::username(),
      hostname: whoami::fallible::hostname().unwrap(),
      distro: whoami::distro(),
      kernel_version: system.kernel_version().unwrap(),
      architecture: uname_info.machine,
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::MachineInfoKernelVersionClick => clipboard::write(self.kernel_version.to_string()),
      _ => Task::none(),
    }
  }

  pub fn view(&self) -> Element<Message> {
    let user_text = text(self.username.to_string()).color(styles::USER_COLOR);
    let at_text = text("@").color(styles::AT_COLOR);
    let host_text = text(self.hostname.to_string()).color(styles::HOST_COLOR);
    let user_at_host = row![user_text, at_text, host_text];

    let distro_text = text(self.distro.to_string()).color(styles::DISTRO_COLOR);
    let kernel_version_text = text(self.kernel_version.to_string()).color(styles::KERNEL_VERSION_COLOR);
    let kernel_version_copy = mouse_area(kernel_version_text)
      .interaction(Interaction::Copy)
      .on_press(Message::MachineInfoKernelVersionClick);
    let architecture_text = text(self.architecture.to_string()).color(styles::ARCHITECTURE_COLOR);

    let content = column![space_row![row![
      user_at_host,
      distro_text,
      kernel_version_copy,
      architecture_text,
    ]]]
    .align_x(Horizontal::Center);
    container(content).center_x(Length::Fill).into()
  }
}
