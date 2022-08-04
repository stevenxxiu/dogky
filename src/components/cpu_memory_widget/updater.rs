use gtk::glib;
use gtk::prelude::ObjectExt;
use std::sync::Arc;
use sysinfo::{
  ComponentExt, CpuExt, CpuRefreshKind, ProcessExt, ProcessRefreshKind, ProcessStatus, RefreshKind, System, SystemExt,
};

use crate::components::CpuMemoryWidget;
use crate::config::CpuMemoryProps;
use crate::utils;

pub struct CpuMemoryWidgetUpdater {
  sysinfo_system: Arc<System>,
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
    let updater = CpuMemoryWidgetUpdater { sysinfo_system };
    updater.update_static_props(cpu_memory_widget);
    updater.update(props, &cpu_memory_widget);
  }

  fn update_static_props(&self, cpu_memory_widget: &CpuMemoryWidget) {
    let mut cpu_model = self.sysinfo_system.global_cpu_info().brand().to_string();
    for &s in CPU_MODEL_REMOVE {
      cpu_model = cpu_model.replace(s, "");
    }
    cpu_memory_widget.set_property("cpu-model", cpu_model);
  }

  fn update_cpu(system: &mut System, cpu_memory_widget: &CpuMemoryWidget) {
    system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency().with_cpu_usage());
    system.refresh_components_list(); // Includes the CPU temperature

    let cpu_frequency = system.global_cpu_info().frequency() as f64 / 1000.0;
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
    CpuMemoryWidgetUpdater::update_cpu(system_mut, cpu_memory_widget);
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
