use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use copypasta::{ClipboardContext, ClipboardProvider};
use freya::prelude::*;
use lazy_static::lazy_static;

use regex::Regex;
use sysinfo::{Disk, DiskRefreshKind, Disks};

use crate::config::DiskConfig;
use crate::format_size::format_size;
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

fn get_disk<'a>(disks: &'a Disks, mount_point: &'a str) -> &'a Disk {
  disks
    .into_iter()
    .find(|disk| disk.mount_point().to_str().unwrap() == mount_point)
    .unwrap()
}

fn get_disk_data(disks: &mut Disks, device_path: &str, mount_point: &str) -> DiskData {
  let mut res = DiskData::default();
  disks.refresh_specifics(true, DiskRefreshKind::nothing().with_storage());

  let disk = get_disk(disks, mount_point);

  res.temperature = get_disk_temperature(device_path);
  res.available_space = disk.available_space();
  res
}

const DISK_DECIMAL_PLACES: usize = 2usize;

#[allow(non_snake_case)]
#[component]
pub fn DiskComponent() -> Element {
  let config = use_context::<DiskConfig>();
  let styles = use_context::<DiskStyles>();
  let global_styles = use_context::<GlobalStyles>();
  let mut ctx = ClipboardContext::new().unwrap();

  let refresh_kind = DiskRefreshKind::nothing().with_storage();
  let mut disks = Disks::new_with_refreshed_list_specifics(refresh_kind);

  let disk = get_disk(&disks, &config.mount_point);

  let model = get_disk_model(&config.device_path).unwrap();

  let file_system_name = disk.file_system().to_str().unwrap();
  let file_system_name = format!("{} ({})", &config.name, &file_system_name);

  let total_space = disk.total_space();

  let mut data = use_signal(DiskData::default);
  let mut used_space = use_signal(|| 0u64);

  use_hook(move || {
    spawn(async move {
      loop {
        data.set(get_disk_data(&mut disks, &config.device_path, &config.mount_point));
        used_space.set(total_space - data().available_space);
        tokio::time::sleep(std::time::Duration::from_secs(config.update_interval)).await;
      }
    })
  });

  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      spacing: global_styles.h_gap.to_string(),
      label { "Disk" },
      CursorArea {
        icon: CursorIcon::Copy,
        label {
          color: styles.name_color.clone(),
          onclick: move |_| { let _ = ctx.set_contents(model.clone()); },
          "{model}"
        },
      }
      label { text_align: "right", color: styles.value_color.clone(), "{data().temperature:.0}°C" },
    }
    rect {
      width: "100%",
      direction: "horizontal",
      spacing: global_styles.h_gap.to_string(),
      label { color: styles.name_color.clone(), "{file_system_name}" },
      label {
        text_align: "right",
        color: styles.value_color.clone(),
        "{format_size(used_space(), DISK_DECIMAL_PLACES): >8}"
        " + {format_size(data().available_space, DISK_DECIMAL_PLACES): >8}"
      },
    }
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: global_styles.h_gap.to_string(),
      cross_align: "center",
      label { color: styles.value_color.clone(), "{format_size(total_space, DISK_DECIMAL_PLACES)}" },
      rect {
        width: "flex(1)",
        height: styles.bar_height.to_string(),
        border: styles.bar_border.clone(),
        rect {
          width: "{used_space() as f32 / total_space as f32 * 100.}%",
          height: "100%",
          background: styles.bar_fill_color.clone(),
        }
      }
    }
  )
}
