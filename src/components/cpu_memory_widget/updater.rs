use gtk::gdk::RGBA;
use gtk::glib::{MainContext, Sender, PRIORITY_DEFAULT};
use gtk::prelude::{BoxExt, Cast, DrawingAreaExt, ObjectExt, WidgetExt};
use gtk::{glib, DrawingArea, Orientation};
use std::iter::zip;
use std::sync::Arc;
use sysinfo::{
  ComponentExt, CpuExt, CpuRefreshKind, ProcessExt, ProcessRefreshKind, ProcessStatus, RefreshKind, System, SystemExt,
};

use crate::components::CpuMemoryWidget;
use crate::config::{CpuBarsProps, CpuMemoryProps};
use crate::utils;

pub struct CpuMemoryWidgetUpdater {
  sysinfo_system: Arc<System>,
  cpu_bar_senders: Vec<Sender<f32>>,
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)"];

impl CpuMemoryWidgetUpdater {
  pub fn init(props: CpuMemoryProps, cpu_memory_widget: &CpuMemoryWidget) {
    let props = Arc::new(props);
    let refresh_kind = RefreshKind::new()
      .with_cpu(CpuRefreshKind::new().with_frequency().with_cpu_usage())
      .with_memory()
      .with_components_list();
    let sysinfo_system = Arc::new(System::new_with_specifics(refresh_kind));
    let num_cpus = sysinfo_system.cpus().len();
    let cpu_bar_senders = CpuMemoryWidgetUpdater::create_cpu_bars(num_cpus, &props.cpu_bars, cpu_memory_widget);
    let updater = CpuMemoryWidgetUpdater {
      sysinfo_system,
      cpu_bar_senders,
    };
    updater.update_static_props(cpu_memory_widget);
    updater.update(props, &cpu_memory_widget);
  }

  fn find_cpu_bars_container(cpu_memory_widget: &CpuMemoryWidget) -> gtk::Box {
    let mut widget = cpu_memory_widget.first_child().unwrap();
    while widget.widget_name() != "cpu-bars-container" {
      widget = widget.next_sibling().unwrap();
    }
    widget.downcast::<gtk::Box>().unwrap()
  }

  fn create_cpu_bars(num_cpus: usize, props: &CpuBarsProps, cpu_memory_widget: &CpuMemoryWidget) -> Vec<Sender<f32>> {
    let container = CpuMemoryWidgetUpdater::find_cpu_bars_container(cpu_memory_widget);
    let mut row = gtk::Box::new(Orientation::Horizontal, props.margin as i32);
    let border_color = RGBA::parse(&props.border_color).unwrap();
    let fill_color = RGBA::parse(&props.fill_color).unwrap();
    let mut senders = vec![];
    for i in 0..num_cpus {
      let bar = DrawingArea::new();
      bar.set_content_width(props.width as i32);
      bar.set_content_height(props.height as i32);
      row.append(&bar);

      let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
      crate::custom_components::build_bar(bar, receiver, &border_color, &fill_color);
      senders.push(sender);

      if ((i + 1) % props.num_per_row == 0) || i == num_cpus - 1 {
        container.append(&row);
        row = gtk::Box::new(Orientation::Horizontal, props.margin as i32);
      }
    }
    senders
  }

  fn update_static_props(&self, cpu_memory_widget: &CpuMemoryWidget) {
    let mut cpu_model = self.sysinfo_system.global_cpu_info().brand().to_string();
    for &s in CPU_MODEL_REMOVE {
      cpu_model = cpu_model.replace(s, "");
    }
    cpu_memory_widget.set_property("cpu-model", cpu_model);
  }

  fn update_cpu(system: &mut System, cpu_memory_widget: &CpuMemoryWidget, cpu_bar_senders: &Vec<Sender<f32>>) {
    system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency().with_cpu_usage());
    system.refresh_components_list(); // Includes the CPU temperature

    let cpu_frequency = system.global_cpu_info().frequency() as f32 / 1000.0;
    cpu_memory_widget.set_property("cpu-frequency", format!("{:.2} GHz", cpu_frequency));

    let cpu_temperature = system
      .components()
      .into_iter()
      .find(|component| component.label().eq("Package id 0"))
      .unwrap()
      .temperature();
    cpu_memory_widget.set_property("cpu-temperature", format!("{}°C", cpu_temperature));

    let cpu_usage = system.global_cpu_info().cpu_usage();
    cpu_memory_widget.set_property("cpu-usage", format!("{:.1}%", cpu_usage));

    zip(system.cpus(), cpu_bar_senders).for_each(|(cpu, sender)| sender.send(cpu.cpu_usage() / 100f32).unwrap());
  }

  fn update_system(system: &System, cpu_memory_widget: &CpuMemoryWidget) {
    cpu_memory_widget.set_property("system-uptime", utils::format_duration(system.uptime()));
  }

  fn update_processes(system: &mut System, cpu_memory_widget: &CpuMemoryWidget) {
    system.refresh_processes_specifics(ProcessRefreshKind::new().with_cpu());

    let processes = system.processes();
    let num_running = processes
      .values()
      .into_iter()
      .filter(|process| process.status() == ProcessStatus::Run)
      .count();
    cpu_memory_widget.set_property(
      "system-num-processes",
      format!("{} / {: >4}", num_running, processes.len()),
    );
  }

  fn update(mut self, props: Arc<CpuMemoryProps>, cpu_memory_widget: &CpuMemoryWidget) {
    let system_mut = Arc::get_mut(&mut self.sysinfo_system).unwrap();
    CpuMemoryWidgetUpdater::update_cpu(system_mut, cpu_memory_widget, &self.cpu_bar_senders);
    CpuMemoryWidgetUpdater::update_system(system_mut, cpu_memory_widget);
    CpuMemoryWidgetUpdater::update_processes(system_mut, cpu_memory_widget);
    glib::source::timeout_add_seconds_local_once(
      props.update_interval,
      glib::clone!(@strong cpu_memory_widget => move || {
        self.update(props, &cpu_memory_widget);
      }),
    );
  }
}
