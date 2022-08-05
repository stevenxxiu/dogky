use gtk::gdk::RGBA;
use gtk::glib::{MainContext, Sender, PRIORITY_DEFAULT};
use gtk::prelude::{BoxExt, DrawingAreaExt};
use gtk::{glib, Builder, DrawingArea, Orientation};
use humansize::FileSize;
use regex::Regex;
use std::iter::zip;
use std::sync::Arc;
use sysinfo::{
  ComponentExt, CpuExt, CpuRefreshKind, ProcessExt, ProcessRefreshKind, ProcessStatus, RefreshKind, System, SystemExt,
};

use crate::config::{CpuBarsProps, CpuMemoryGraphContainerProps, CpuMemoryProps};
use crate::custom_components::build_graph;
use crate::gtk_utils::set_label;
use crate::utils;
use crate::utils::FILE_SIZE_OPTS;

pub struct CpuMemoryWidget {
  sysinfo_system: Arc<System>,
  cpu_bar_senders: Vec<Sender<f32>>,
  cpu_graph_sender: Sender<f32>,
  memory_graph_sender: Sender<f32>,
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)"];

impl CpuMemoryWidget {
  pub fn build(props: CpuMemoryProps) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/cpu_memory_widget.ui");
    let container: gtk::Box = builder.object("cpu_memory_widget").unwrap();

    let props = Arc::new(props);
    let refresh_kind = RefreshKind::new()
      .with_cpu(CpuRefreshKind::new().with_frequency().with_cpu_usage())
      .with_memory()
      .with_components_list();
    let sysinfo_system = System::new_with_specifics(refresh_kind);

    let num_cpus = sysinfo_system.cpus().len();
    let cpu_bar_senders = CpuMemoryWidget::build_cpu_bars(num_cpus, &props.cpu_bars, &builder);
    let [cpu_graph_sender, memory_graph_sender] =
      CpuMemoryWidget::build_graphs(&sysinfo_system, &props.graphs, &builder);
    CpuMemoryWidget::update_static_props(&sysinfo_system, &builder);

    let updater = CpuMemoryWidget {
      sysinfo_system: Arc::new(sysinfo_system),
      cpu_bar_senders,
      cpu_graph_sender,
      memory_graph_sender,
    };
    updater.update(props, &builder);
    container
  }

  fn build_cpu_bars(num_cpus: usize, props: &CpuBarsProps, builder: &Builder) -> Vec<Sender<f32>> {
    let container = builder.object::<gtk::Box>("cpu_bars_container").unwrap();
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

  fn build_graphs(
    sysinfo_system: &System,
    props: &CpuMemoryGraphContainerProps,
    builder: &Builder,
  ) -> [Sender<f32>; 2] {
    let container = builder.object::<gtk::Box>("cpu_memory_graph_container").unwrap();
    container.set_spacing(props.margin as i32);
    [
      (&props.cpu, 100f32),
      (&props.memory, sysinfo_system.total_memory() as f32),
    ]
    .map(|(graph_props, max_value)| {
      let graph = DrawingArea::new();
      graph.set_content_width(props.width as i32);
      graph.set_content_height(props.height as i32);
      container.append(&graph);

      let border_color = RGBA::parse(&graph_props.border_color).unwrap();
      let fill_color = RGBA::parse(&graph_props.fill_color).unwrap();
      let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
      build_graph(graph, receiver, max_value, &border_color, &fill_color);
      sender
    })
  }

  fn update_static_props(sysinfo_system: &System, builder: &Builder) {
    let mut cpu_model = sysinfo_system.global_cpu_info().brand().to_string();
    for &s in CPU_MODEL_REMOVE {
      cpu_model = cpu_model.replace(s, "");
    }
    set_label(builder, "cpu_model", &cpu_model);

    let lshw_output = std::fs::read_to_string("/run/lshw-memory.txt").unwrap();
    let lshw_regex = Regex::new(r"\d+ MHz").unwrap();
    let memory_frequency = lshw_regex.find(&lshw_output).into_iter().next().unwrap().as_str();
    set_label(builder, "memory_frequency", memory_frequency);
  }

  fn update_cpu(
    system: &mut System,
    builder: &Builder,
    cpu_bar_senders: &Vec<Sender<f32>>,
    cpu_graph_sender: &Sender<f32>,
  ) {
    system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency().with_cpu_usage());
    system.refresh_components_list(); // Includes the CPU temperature

    let cpu_frequency = system.global_cpu_info().frequency() as f32 / 1000.0;
    set_label(builder, "cpu_frequency", &format!("{:.2} GHz", cpu_frequency));

    let cpu_temperature = system
      .components()
      .into_iter()
      .find(|component| component.label().eq("Package id 0"))
      .unwrap()
      .temperature();
    set_label(builder, "cpu_temperature", &format!("{}°C", cpu_temperature));

    let cpu_usage = system.global_cpu_info().cpu_usage();
    set_label(builder, "cpu_usage", &format!("{:.1}%", cpu_usage));

    zip(system.cpus(), cpu_bar_senders).for_each(|(cpu, sender)| sender.send(cpu.cpu_usage() / 100f32).unwrap());
    cpu_graph_sender.send(system.global_cpu_info().cpu_usage()).unwrap();
  }

  fn update_memory(system: &mut System, builder: &Builder, memory_graph_sender: &Sender<f32>) {
    system.refresh_memory();

    let used_memory = system.used_memory() * 1024;
    let total_memory = system.total_memory() * 1024;
    let memory_usage_str = format!(
      "{: >8}/{: >8} = {: >3.0}%",
      used_memory.file_size(FILE_SIZE_OPTS).unwrap(),
      total_memory.file_size(FILE_SIZE_OPTS).unwrap(),
      (used_memory as f32) / (total_memory as f32) * 100.0
    );
    set_label(builder, "memory_usage", &memory_usage_str);

    memory_graph_sender.send(system.used_memory() as f32).unwrap();
  }

  fn update_system(system: &System, builder: &Builder) {
    set_label(builder, "system_uptime", &utils::format_duration(system.uptime()));
  }

  fn update_processes(system: &mut System, builder: &Builder) {
    system.refresh_processes_specifics(ProcessRefreshKind::new().with_cpu());

    let processes = system.processes();
    let num_running = processes
      .values()
      .into_iter()
      .filter(|process| process.status() == ProcessStatus::Run)
      .count();
    let system_num_processes = format!("{} / {: >4}", num_running, processes.len());
    set_label(builder, "system_num_processes", &system_num_processes);
  }

  fn update(mut self, props: Arc<CpuMemoryProps>, builder: &Builder) {
    let system_mut = Arc::get_mut(&mut self.sysinfo_system).unwrap();
    CpuMemoryWidget::update_cpu(system_mut, builder, &self.cpu_bar_senders, &self.cpu_graph_sender);
    CpuMemoryWidget::update_memory(system_mut, builder, &self.memory_graph_sender);
    CpuMemoryWidget::update_system(system_mut, builder);
    CpuMemoryWidget::update_processes(system_mut, builder);
    glib::source::timeout_add_seconds_local_once(
      props.update_interval,
      glib::clone!(@strong builder => move || {
        self.update(props, &builder);
      }),
    );
  }
}
