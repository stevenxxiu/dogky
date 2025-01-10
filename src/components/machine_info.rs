use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{column, container, mouse_area, row, text};
use iced::{clipboard, Task};
use iced::{Element, Length};
use sysinfo::System;

use crate::message::{MachineInfoMessage, Message};
use crate::styles_config::{GlobalStyles, MachineInfoStyles};
use crate::ui_utils::WithSpacing;

pub struct MachineInfoComponent {
  global_styles: GlobalStyles,
  styles: MachineInfoStyles,
  username: String,
  hostname: String,
  distro: String,
  kernel_version: String,
  architecture: String,
}

impl MachineInfoComponent {
  pub fn new(global_styles: GlobalStyles, styles: MachineInfoStyles) -> Self {
    let uname_info = uname::uname().unwrap();
    Self {
      global_styles,
      styles,
      username: whoami::username(),
      hostname: whoami::fallible::hostname().unwrap(),
      distro: whoami::distro(),
      kernel_version: System::kernel_version().unwrap(),
      architecture: uname_info.machine,
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    if let Message::MachineInfo(message) = message {
      return match message {
        MachineInfoMessage::KernelVersionClick => clipboard::write(self.kernel_version.to_string()),
      };
    }
    Task::none()
  }

  pub fn view(&self) -> Element<Message> {
    let global_styles = &self.global_styles;
    let styles = &self.styles;
    let row_style = WithSpacing::new(global_styles.h_gap);

    let user_text = text(self.username.to_string()).color(styles.user_color.clone());
    let at_text = text("@").color(styles.at_color.clone());
    let host_text = text(self.hostname.to_string()).color(styles.host_color.clone());
    let user_at_host = row![user_text, at_text, host_text];

    let distro_text = text(self.distro.to_string()).color(styles.distro_color.clone());
    let kernel_version_text = text(self.kernel_version.to_string()).color(styles.kernel_version_color.clone());
    let kernel_version_copy = mouse_area(kernel_version_text)
      .interaction(Interaction::Copy)
      .on_press(Message::MachineInfo(MachineInfoMessage::KernelVersionClick));
    let architecture_text = text(self.architecture.to_string()).color(styles.architecture_color.clone());

    let content = column![row_style.row(row![user_at_host, distro_text, kernel_version_copy, architecture_text,])]
      .align_x(Horizontal::Center);
    container(content).center_x(Length::Fill).into()
  }
}
