use gtk::gdk::RGBA;
use gtk::glib::{MainContext, Sender, PRIORITY_DEFAULT};
use gtk::prelude::{BoxExt, DrawingAreaExt, WidgetExt};
use gtk::{glib, Align, Builder, DrawingArea};
use lazy_static::lazy_static;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use sysinfo::{Disk, DiskExt, RefreshKind, System, SystemExt};

use crate::config::{DiskBarProps, DiskProps};
use crate::format_size::format_size;
use crate::gtk_utils::{set_copyable_label, set_label};

// We can't use `Disk` directly. The reason probably is as there's the field `.available_space`.
#[derive(Clone)]
pub struct DiskWidget {
  builder: Arc<Builder>,
  file_system_bar_sender: Sender<f32>,
}

const BAR_MARGIN_LEFT: u32 = 90;
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

impl DiskWidget {
  pub fn build(props: DiskProps, container_width: u32) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/disk_widget.ui");
    let container: gtk::Box = builder.object("disk_widget").unwrap();

    let props = Arc::new(props);
    let refresh_kind = RefreshKind::new().with_disks_list().with_disks();
    let system = System::new_with_specifics(refresh_kind);

    let file_system_bar_sender = DiskWidget::build_file_system_bar(&props.bar, container_width, &builder);

    let updater = DiskWidget {
      builder: Arc::new(builder),
      file_system_bar_sender,
    };
    updater.update_static_props(&system, &props);
    updater.update(Arc::new(system), props);
    container
  }

  fn get_disk<'a>(system: &'a System, mount_point: &'a str) -> &'a Disk {
    system
      .disks()
      .into_iter()
      .find(|disk| disk.mount_point().to_str().unwrap() == mount_point)
      .unwrap()
  }

  fn build_file_system_bar(props: &DiskBarProps, container_width: u32, builder: &Builder) -> Sender<f32> {
    let container = builder.object::<gtk::Box>("file_system_bar_container").unwrap();
    let border_color = RGBA::parse(&props.border_color).unwrap();
    let fill_color = RGBA::parse(&props.fill_color).unwrap();
    let bar = DrawingArea::new();
    bar.set_content_width((container_width - BAR_MARGIN_LEFT) as i32);
    bar.set_content_height(props.height as i32);
    bar.set_hexpand(true);
    bar.set_halign(Align::End);
    container.append(&bar);

    let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
    crate::custom_components::build_bar(bar, receiver, &border_color, &fill_color);
    sender
  }

  fn update_static_props(&self, system: &System, props: &DiskProps) {
    let disk = DiskWidget::get_disk(system, &props.mount_point);

    let disk_model = get_disk_model(&props.device_path).unwrap();
    set_copyable_label(&self.builder, "disk_model", disk_model);

    let file_system_name = std::str::from_utf8(disk.file_system()).unwrap().to_string();
    let file_system_name = format!("{} ({})", &props.name, &file_system_name);
    set_label(&self.builder, "file_system_name", &file_system_name);
  }

  fn update_disk(&self, system: &mut System, props: &DiskProps) {
    system.refresh_disks();
    let disk = DiskWidget::get_disk(&system, &props.mount_point);

    let disk_temperature = get_disk_temperature(&props.device_path);
    set_label(&self.builder, "disk_temperature", &format!("{:.0}°C", disk_temperature));

    let total_space = disk.total_space();
    set_label(
      &self.builder,
      "file_system_total",
      &format_size(total_space, DISK_DECIMAL_PLACES),
    );

    let available_space = disk.available_space();
    let used_space = total_space - available_space;
    let file_system_usage = format!(
      "{: >8} + {: >8}",
      &format_size(total_space, DISK_DECIMAL_PLACES),
      &format_size(available_space, DISK_DECIMAL_PLACES),
    );
    set_label(&self.builder, "file_system_usage", &file_system_usage);

    self
      .file_system_bar_sender
      .send(used_space as f32 / total_space as f32)
      .unwrap();
  }

  fn update(&self, mut system: Arc<System>, props: Arc<DiskProps>) {
    let system_deref = Arc::get_mut(&mut system).unwrap();
    system_deref.refresh_networks();

    self.update_disk(system_deref, &props);

    let self_clone = self.clone();
    glib::source::timeout_add_seconds_local_once(props.update_interval, move || self_clone.update(system, props));
  }
}
