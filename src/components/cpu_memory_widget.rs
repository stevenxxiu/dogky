use enum_map::{enum_map, Enum, EnumMap};
use gtk::gdk::RGBA;
use gtk::glib::{MainContext, Sender, PRIORITY_DEFAULT};
use gtk::pango::EllipsizeMode;
use gtk::prelude::{BoxExt, DrawingAreaExt, GestureExt, WidgetExt};
use gtk::{glib, Builder, DrawingArea, Label, Orientation};
use lazy_static::lazy_static;
use regex::Regex;
use std::iter::zip;
use std::process::Command;
use std::sync::Arc;
use sysinfo::{
  ComponentExt, Cpu, CpuExt, CpuRefreshKind, Process, ProcessExt, ProcessRefreshKind, ProcessStatus, RefreshKind,
  System, SystemExt,
};

use crate::config::{CpuBarsProps, CpuMemoryGraphContainerProps, CpuMemoryProcessListProps, CpuMemoryProps};
use crate::custom_components::build_graph;
use crate::format_size::format_size;
use crate::gtk_utils::{set_copyable_label, set_label};
use crate::serde_structs::SerializableCommand;
use crate::utils;

const MEMORY_DECIMAL_PLACES: usize = 1usize;

#[derive(Enum, PartialEq, strum_macros::Display, Debug)]
enum ProcessSortBy {
  #[strum(serialize = "cpu")]
  CPU,
  #[strum(serialize = "memory")]
  Memory,
}

#[derive(Enum, PartialEq, strum_macros::Display, Debug)]
enum ProcessColumn {
  #[strum(serialize = "command")]
  Command,
  #[strum(serialize = "pid")]
  PID,
  #[strum(serialize = "cpu")]
  CPU,
  #[strum(serialize = "memory")]
  Memory,
}

#[derive(Clone)]
pub struct CpuMemoryWidget {
  builder: Arc<Builder>,
  cpu_bar_senders: Vec<Sender<f32>>,
  cpu_graph_sender: Sender<f32>,
  memory_graph_sender: Sender<f32>,
  grouped_process_labels: EnumMap<ProcessSortBy, EnumMap<ProcessColumn, Vec<Label>>>,
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)"];

impl CpuMemoryWidget {
  pub fn build(props: CpuMemoryProps, container_width: u32) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/cpu_memory_widget.ui");
    let container: gtk::Box = builder.object("cpu_memory_widget").unwrap();

    let props = Arc::new(props);
    let refresh_kind = RefreshKind::new()
      .with_cpu(CpuRefreshKind::new().with_frequency().with_cpu_usage())
      .with_memory()
      .with_components_list();
    let system = System::new_with_specifics(refresh_kind);

    let num_cpus = system.cpus().len();
    let cpu_bar_senders = CpuMemoryWidget::build_cpu_bars(num_cpus, &props.cpu_bars, container_width, &builder);
    let [cpu_graph_sender, memory_graph_sender] =
      CpuMemoryWidget::build_graphs(&system, &props.graphs, container_width, &builder);
    CpuMemoryWidget::add_process_list_click_listener(&props.process_list.top_command, &builder);
    let grouped_process_labels = CpuMemoryWidget::build_process_list(&props.process_list, &builder);

    let updater = CpuMemoryWidget {
      builder: Arc::new(builder),
      cpu_bar_senders,
      cpu_graph_sender,
      memory_graph_sender,
      grouped_process_labels,
    };
    updater.update_static_props(&system);
    updater.update(Arc::new(system), props);
    container
  }

  fn build_cpu_bars(
    num_cpus: usize,
    props: &CpuBarsProps,
    container_width: u32,
    builder: &Builder,
  ) -> Vec<Sender<f32>> {
    let container = builder.object::<gtk::Box>("cpu_bars_container").unwrap();
    let margin = (container_width - props.num_per_row as u32 * props.width) / (props.num_per_row as u32 - 1);
    let mut row = gtk::Box::new(Orientation::Horizontal, margin as i32);
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
        row = gtk::Box::new(Orientation::Horizontal, margin as i32);
      }
    }
    senders
  }

  fn build_graphs(
    system: &System,
    props: &CpuMemoryGraphContainerProps,
    container_width: u32,
    builder: &Builder,
  ) -> [Sender<f32>; 2] {
    let container = builder.object::<gtk::Box>("cpu_memory_graph_container").unwrap();
    let graph_specs = [(&props.cpu, 100f32), (&props.memory, system.total_memory() as f32)];
    let margin = (container_width - graph_specs.len() as u32 * props.width) / (graph_specs.len() as u32 - 1);
    container.set_spacing(margin as i32);
    graph_specs.map(|(graph_props, max_value)| {
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

  fn build_process_list(
    props: &CpuMemoryProcessListProps,
    builder: &Builder,
  ) -> EnumMap<ProcessSortBy, EnumMap<ProcessColumn, Vec<Label>>> {
    let [pid_container, cpu_container, memory_container] = [
      "process_pid_container",
      "process_cpu_container",
      "process_memory_container",
    ]
    .map(|name| builder.object::<gtk::Box>(name).unwrap());
    pid_container.set_width_request(props.pid_width as i32);
    cpu_container.set_width_request(props.cpu_width as i32);
    memory_container.set_width_request(props.memory_width as i32);
    enum_map! {
      sort_by => {
        enum_map! {
          column => {
            let container = builder
              .object::<gtk::Box>(&format!("sort_by_{}_{}_container", sort_by, column))
              .unwrap();
            (0..props.num_processes)
              .map(|_| {
                let label = Label::new(None);
                if column == ProcessColumn::Command {
                  label.set_xalign(0.0);
                  label.set_max_width_chars(1); // Required for ellipsize to work
                  label.set_ellipsize(EllipsizeMode::End);
                } else {
                  label.set_xalign(1.0);
                }
                container.append(&label);
                label
              })
              .collect::<Vec<Label>>()
          }
        }
      }
    }
  }

  fn add_process_list_click_listener(top_command: &SerializableCommand, builder: &Builder) {
    if top_command.is_empty() {
      return;
    }
    let container = builder.object::<gtk::Box>("cpu_memory_process_container").unwrap();
    let gesture = gtk::GestureClick::new();
    let top_command: Vec<String> = top_command
      .iter()
      .map(|part| utils::substitute_env_vars(part))
      .collect();
    gesture.connect_released(move |gesture, _, _, _| {
      gesture.set_state(gtk::EventSequenceState::Claimed);
      let (binary, args) = top_command.split_at(1);
      Command::new(&binary[0]).args(args).status().unwrap();
    });
    container.add_controller(&gesture);
    container.set_cursor_from_name(Option::from("pointer"));
  }

  fn update_static_props(&self, system: &System) {
    lazy_static! {
      static ref RE_FREQUENCY: Regex = Regex::new(r"\d+ MHz").unwrap();
    }
    let mut cpu_model = system.global_cpu_info().brand().to_string();
    for &s in CPU_MODEL_REMOVE {
      cpu_model = cpu_model.replace(s, "");
    }
    set_copyable_label(&self.builder, "cpu_model", cpu_model);

    let lshw_output = std::fs::read_to_string("/run/lshw-memory.txt").unwrap();
    let memory_frequency = RE_FREQUENCY.find(&lshw_output).into_iter().next().unwrap().as_str();
    set_label(&self.builder, "memory_frequency", memory_frequency);
  }

  fn update_cpu(&self, system: &mut System) {
    system.refresh_cpu_specifics(CpuRefreshKind::new().with_frequency().with_cpu_usage());
    system.refresh_components_list(); // Includes the CPU temperature

    let cpu_frequency = system.global_cpu_info().frequency() as f32 / 1000.0;
    set_label(&self.builder, "cpu_frequency", &format!("{:.2} GHz", cpu_frequency));

    let cpu_temperature = system
      .components()
      .into_iter()
      .find(|component| component.label().eq("Package id 0"))
      .unwrap()
      .temperature();
    set_label(&self.builder, "cpu_temperature", &format!("{}°C", cpu_temperature));

    let cpu_usage = system.global_cpu_info().cpu_usage();
    set_label(&self.builder, "cpu_usage", &format!("{:.1}%", cpu_usage));

    zip::<&[Cpu], &[Sender<f32>]>(system.cpus(), self.cpu_bar_senders.as_ref())
      .for_each(|(cpu, sender)| sender.send(cpu.cpu_usage() / 100f32).unwrap());
    self
      .cpu_graph_sender
      .send(system.global_cpu_info().cpu_usage())
      .unwrap();
  }

  fn update_memory(&self, system: &mut System) {
    system.refresh_memory();

    let used_memory = system.used_memory() * 1024;
    let total_memory = system.total_memory() * 1024;
    let memory_usage_str = format!(
      "{: >8}/{: >8} = {: >3.0}%",
      format_size(used_memory, MEMORY_DECIMAL_PLACES),
      format_size(total_memory, MEMORY_DECIMAL_PLACES),
      (used_memory as f32) / (total_memory as f32) * 100.0
    );
    set_label(&self.builder, "memory_usage", &memory_usage_str);

    self.memory_graph_sender.send(system.used_memory() as f32).unwrap();
  }

  fn update_system(&self, system: &System) {
    set_label(&self.builder, "system_uptime", &utils::format_duration(system.uptime()));
  }

  fn update_processes(&self, system: &mut System, props: &CpuMemoryProps) {
    system.refresh_processes_specifics(ProcessRefreshKind::new().with_cpu());
    let mut processes: Vec<&Process> = system.processes().into_iter().map(|(_pid, process)| process).collect();

    let num_running = processes
      .iter()
      .filter(|&process| process.status() == ProcessStatus::Run)
      .count();
    let system_num_processes = format!("{} / {: >4}", num_running, system.processes().len());
    set_label(&self.builder, "system_num_processes", &system_num_processes);

    let update_grouped_processes = |processes: &Vec<&Process>, labels: &EnumMap<ProcessColumn, Vec<Label>>| {
      let num_cpus = system.cpus().len();
      for (i, &process) in processes[..props.process_list.num_processes].iter().enumerate() {
        // For simplicity, the command is just joined with spaces, and not escaped
        let args = process
          .cmd()
          .iter()
          .skip(1)
          .fold(String::new(), |res, cur| res + cur.as_str() + " ");
        labels[ProcessColumn::Command][i].set_label(&format!("{} {}", process.name(), args));
        labels[ProcessColumn::PID][i].set_label(&process.pid().to_string());
        labels[ProcessColumn::CPU][i].set_label(&format!("{:.2}", process.cpu_usage() / num_cpus as f32));
        let memory_bytes = process.memory() * 1024;
        labels[ProcessColumn::Memory][i].set_label(&format_size(memory_bytes, MEMORY_DECIMAL_PLACES));
      }
    };

    processes.sort_by(|&process_1, &process_2| process_2.cpu_usage().partial_cmp(&process_1.cpu_usage()).unwrap());
    update_grouped_processes(&processes, &self.grouped_process_labels[ProcessSortBy::CPU]);

    processes.sort_by(|&process_1, &process_2| process_2.memory().partial_cmp(&process_1.memory()).unwrap());
    update_grouped_processes(&processes, &self.grouped_process_labels[ProcessSortBy::Memory]);
  }

  fn update(&self, mut system: Arc<System>, props: Arc<CpuMemoryProps>) {
    let system_deref = Arc::get_mut(&mut system).unwrap();
    system_deref.refresh_networks();

    self.update_cpu(system_deref);
    self.update_memory(system_deref);
    self.update_system(system_deref);
    self.update_processes(system_deref, &props.as_ref());

    let self_clone = self.clone();
    glib::source::timeout_add_seconds_local_once(props.update_interval, move || self_clone.update(system, props));
  }
}
