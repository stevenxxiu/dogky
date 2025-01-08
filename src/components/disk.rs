use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{canvas, column, container, mouse_area, row, text, Text};
use iced::{clipboard, time, Element, Length, Subscription, Task};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use sysinfo::{Disk, DiskRefreshKind, Disks};

use crate::config::DiskProps;
use crate::custom_components::Bar;
use crate::format_size::format_size;
use crate::message::Message;
use crate::styles::disk as styles;
use crate::ui_utils::{expand_right, space_row};

pub struct DiskComponent {
  config_props: DiskProps,
  container_width: f32,
  disks: Disks,
  model: String,
  file_system_name: String,
  total_space: u64,
  live: DiskLiveProps,
}

#[derive(Default)]
struct DiskLiveProps {
  temperature: f32,
  available_space: u64,
}

const DISK_DECIMAL_PLACES: usize = 2usize;

fn get_disk_model(device_path: &str) -> Result<String, String> {
  lazy_static! {
    static ref RE_ID_MODEL: Regex = Regex::new(r"^ID_MODEL=(.+)").unwrap();
  }
  let stdout = Command::new("udevadm")
    .args(["info", "--query=property", &format!("--name={}", device_path)])
    .output()
    .unwrap()
    .stdout;
  for line in stdout.lines() {
    if let Some(captures) = RE_ID_MODEL.captures(&line.unwrap()) {
      let disk_model: String = captures.get(1).unwrap().as_str().to_string();
      let disk_model = disk_model.replace("_", " ");
      return Ok(disk_model);
    }
  }
  Err(format!("Specified device path {} does not exist", device_path))
}

fn get_disk_temperature(device_path: &str) -> f32 {
  lazy_static! {
    static ref RE_TEMPERATURE: Regex = Regex::new(r"^(\d+)\d{3}$").unwrap();
  }
  let device_name = Path::new(device_path).file_name().unwrap().to_str().unwrap();
  let path = format!("/sys/class/block/{}/device/hwmon/hwmon1/temp1_input", device_name);
  let file = File::open(path).unwrap();
  let line = BufReader::new(file).lines().next().unwrap().unwrap();
  line.parse::<i32>().unwrap() as f32 / 1000.0
}

impl DiskComponent {
  pub fn new(config_props: DiskProps, container_width: f32) -> Self {
    let refresh_kind = DiskRefreshKind::nothing().with_storage();
    let disks = Disks::new_with_refreshed_list_specifics(refresh_kind);

    let disk = DiskComponent::get_disk(&disks, &config_props.mount_point);

    let model = get_disk_model(&config_props.device_path).unwrap();

    let file_system_name = disk.file_system().to_str().unwrap();
    let file_system_name = format!("{} ({})", &config_props.name, &file_system_name);

    let total_space = disk.total_space();

    Self {
      config_props,
      container_width,
      disks,
      model,
      file_system_name,
      total_space,
      live: DiskLiveProps::default(),
    }
  }

  fn get_disk<'a>(disks: &'a Disks, mount_point: &'a str) -> &'a Disk {
    disks
      .into_iter()
      .find(|disk| disk.mount_point().to_str().unwrap() == mount_point)
      .unwrap()
  }

  fn update_data(&mut self) {
    let props = &self.config_props;
    let live = &mut self.live;
    let disks = &mut self.disks;
    disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage());
    let disk = DiskComponent::get_disk(disks, &props.mount_point);

    live.temperature = get_disk_temperature(&props.device_path);
    live.available_space = disk.available_space();
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::DiskTick => {
        self.update_data();
        Task::none()
      }
      Message::DiskModelClick => clipboard::write(self.model.to_string()),
      _ => Task::none(),
    }
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let props = &self.config_props;
    time::every(Duration::from_secs(props.update_interval)).map(|_instant| Message::DiskTick)
  }

  pub fn view(&self) -> Element<Message> {
    let live = &self.live;

    let name_text = |s: String| -> Text { text(s).color(styles::NAME_COLOR) };
    let value_text = |s: String| -> Text { text(s).color(styles::VALUE_COLOR) };

    let model_text = name_text(self.model.to_string());
    let model_copy = mouse_area(model_text)
      .interaction(Interaction::Copy)
      .on_press(Message::DiskModelClick);

    let used_space = self.total_space - live.available_space;
    let file_system_usage = format!(
      "{: >8} + {: >8}",
      format_size(self.total_space, DISK_DECIMAL_PLACES),
      &format_size(live.available_space, DISK_DECIMAL_PLACES),
    );

    let bar = Bar {
      value: used_space as f32 / self.total_space as f32,
      width: self.container_width,
      height: styles::BAR_HEIGHT,
      fill_color: styles::BAR_FILL_COLOR,
      border_color: styles::BAR_BORDER_COLOR,
      ..Default::default()
    };

    column![
      row![
        space_row![row![text("Disk"), model_copy]],
        expand_right![value_text(format!("{:.0}°C", live.temperature))]
      ],
      row![
        name_text(self.file_system_name.to_string()),
        expand_right![value_text(file_system_usage)]
      ],
      container(canvas(bar).width(self.container_width).height(styles::BAR_HEIGHT)),
    ]
    .width(Length::Fill)
    .into()
  }
}
