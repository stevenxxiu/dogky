use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;
use freya::text_edit::Clipboard;
use futures_lite::stream::StreamExt;
use lazy_static::lazy_static;

use regex::Regex;
use sysinfo::{Disk, DiskRefreshKind, Disks};

use crate::config::DiskConfig;
use crate::format_size::format_size;
use crate::freya_utils::{
  border_fill_width, color_label, cursor_area, flex_cont_factory, horizontal_cont_factory, right_value_label,
  value_label_factory,
};
use crate::styles_config::{DiskStyles, GlobalStyles};

#[derive(Default, Clone, Debug)]
struct DiskData {
  temperature: f32,
  available_space: u64,
}

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

fn get_disk_temperature(temperature_path: &str) -> f32 {
  let file = File::open(temperature_path).unwrap();
  let line = BufReader::new(file).lines().next().unwrap().unwrap();
  line.parse::<i32>().unwrap() as f32 / 1000.0
}

fn get_disk<'a>(disks: &'a Disks, mount_point: &'a str) -> &'a Disk {
  disks
    .into_iter()
    .find(|disk| disk.mount_point().to_str().unwrap() == mount_point)
    .unwrap()
}

fn get_disk_data(disks: &mut Disks, temperature_path: &str, mount_point: &str) -> DiskData {
  let mut res = DiskData::default();
  disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage());

  let disk = get_disk(disks, mount_point);

  res.temperature = get_disk_temperature(temperature_path);
  res.available_space = disk.available_space();
  res
}

const DISK_DECIMAL_PLACES: usize = 2usize;

pub fn disk_component() -> Rect {
  let config = use_consume::<DiskConfig>();
  let styles = use_consume::<DiskStyles>();
  let global_styles = use_consume::<GlobalStyles>();

  let refresh_kind = DiskRefreshKind::nothing().with_storage();
  let mut disks = Disks::new_with_refreshed_list_specifics(refresh_kind);

  let disk = get_disk(&disks, &config.mount_point);

  let model = get_disk_model(&config.device_path).unwrap();

  let file_system_name = disk.file_system().to_str().unwrap();
  let file_system_name = format!("{} ({})", &config.name, &file_system_name);

  let total_space = disk.total_space();

  let mut data = use_state(DiskData::default);
  let mut used_space = use_state(|| 0u64);

  use_hook(|| {
    spawn(async move {
      loop {
        data.set(get_disk_data(&mut disks, &config.temperature_path, &config.mount_point));
        used_space.set(total_space - data.read().available_space);
        Timer::interval(Duration::from_secs(config.update_interval))
          .next()
          .await;
      }
    })
  });

  let horizontal_cont = horizontal_cont_factory(global_styles.h_gap);
  let flex_cont = flex_cont_factory(global_styles.h_gap);
  let value_label = value_label_factory(*styles.value_color);

  rect().children([
    horizontal_cont(vec![
      label().text("Disk").into(),
      cursor_area(CursorIcon::Copy)
        .child(
          color_label(*styles.name_color, model.clone()).on_mouse_down(move |_| Clipboard::set(model.clone()).unwrap()),
        )
        .into(),
      right_value_label(*styles.value_color, format!("{:.0}°C", data.read().temperature)).into(),
    ])
    .into(),
    horizontal_cont(vec![
      color_label(*styles.name_color, file_system_name).into(),
      right_value_label(
        *styles.value_color,
        format!(
          "{: >8} + {: >8}",
          format_size(used_space(), DISK_DECIMAL_PLACES),
          format_size(data.read().available_space, DISK_DECIMAL_PLACES)
        ),
      )
      .into(),
    ])
    .into(),
    flex_cont(vec![value_label(format_size(total_space, DISK_DECIMAL_PLACES)).into()])
      .cross_align(Alignment::Center)
      .child(
        rect()
          .width(Size::flex(1.))
          .height(Size::px(styles.bar_height))
          .border(border_fill_width(*styles.bar_border_color, styles.bar_border_width))
          .child(
            rect()
              .width(Size::percent(used_space() as f32 / total_space as f32 * 100.))
              .height(Size::percent(100.))
              .background(*styles.bar_fill_color),
          ),
      )
      .into(),
  ])
}
