use std::collections::HashSet;
use std::process::Command;

use arboard::Clipboard;
use circular_queue::CircularQueue;
use freya::prelude::*;
use lazy_static::lazy_static;

use regex::Regex;
use sysinfo::{
  Components, CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate,
  RefreshKind, System, UpdateKind,
};

use crate::config::CpuMemoryConfig;
use crate::custom_components::{Graph, LabelRight};
use crate::format_size::format_size;
use crate::styles_config::{CpuMemoryStyles, GlobalStyles};
use crate::utils::{self, format_used, MEMORY_DECIMAL_PLACES};

#[derive(Default, Clone, Debug)]
struct CpuData {
  frequency: f32,
  temperature: f32,
  usage: f32,
  core_usage: Vec<f32>,
}

#[derive(Default, Clone, Debug)]
struct MemoryData {
  memory_usage: u64,
  swap_usage: u64,
}

#[derive(Default, Clone)]
struct ProcessData {
  processes: Vec<ProcessProps>,
  num_running: usize,
}

#[derive(Clone)]
struct ProcessProps {
  cmd: String,
  pid: Pid,
  cpu_usage: f32,
  memory_usage: u64,
}

fn get_cpu_data(system: &mut System, components: &mut Components) -> CpuData {
  let mut res = CpuData::default();
  system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_frequency().with_cpu_usage());

  let cpus = system.cpus();
  res.frequency = cpus[0].frequency() as f32 / 1000.0;

  components.refresh(true);
  res.temperature = components
    .into_iter()
    .find(|component| component.label().eq("Package id 0"))
    .unwrap()
    .temperature()
    .unwrap();

  res.usage = system.global_cpu_usage();
  res.core_usage = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
  res
}

fn get_memory_data(system: &mut System) -> MemoryData {
  let mut res = MemoryData::default();
  system.refresh_memory();
  res.memory_usage = system.used_memory();
  res.swap_usage = system.used_swap();
  res
}

fn get_process_data(system: &mut System) -> ProcessData {
  let mut res = ProcessData::default();
  // Create another `System` from scratch every time, or we end up with more processes than there should be
  let mut system_new = System::new_with_specifics(RefreshKind::nothing());
  system_new.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
  system.refresh_processes_specifics(
    ProcessesToUpdate::All,
    true,
    ProcessRefreshKind::nothing()
      .with_memory()
      .with_cpu()
      .with_cmd(UpdateKind::Always),
  );
  let pid_to_process = system_new.processes();
  let mut task_pids: HashSet<Pid> = HashSet::new();
  for process in pid_to_process.values() {
    if let Some(tasks) = process.tasks() {
      task_pids.extend(tasks)
    }
  }
  res.num_running = 0;
  for (pid, process) in pid_to_process {
    if task_pids.contains(pid) {
      continue;
    }
    if process.status() == ProcessStatus::Run {
      res.num_running += 1;
    }
    if let Some(live_process) = system.process(*pid) {
      let args = live_process
        .cmd()
        .iter()
        .skip(1)
        .fold(String::new(), |res, cur| res + cur.to_str().unwrap() + " ");
      res.processes.push(ProcessProps {
        cmd: format!("{} {}", live_process.name().to_str().unwrap(), args),
        pid: *pid,
        cpu_usage: live_process.cpu_usage(),
        memory_usage: live_process.memory(),
      });
    }
  }
  res
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)", "!"];

#[allow(non_snake_case)]
#[component]
fn CpuBarsComponent(cpu_core_usage: Vec<f32>) -> Element {
  let styles = use_context::<CpuMemoryStyles>();
  rsx!(
    rect {
      direction: "vertical",
      spacing: styles.bars_v_gap.to_string(),
      for i in 0..cpu_core_usage.len().div_ceil(styles.bars_per_row) {
        rect {
          width: "100%",
          direction: "horizontal",
          content: "flex",
          spacing: styles.bar_h_gap.to_string(),
          for j in 0..(cpu_core_usage.len() - i * styles.bars_per_row).min(styles.bars_per_row) {
            rect {
              width: "flex(1)",
              height: styles.bar_height.to_string(),
              border: styles.bar_border.clone(),
              rect {
                width: "{cpu_core_usage[i * styles.bars_per_row + j]}%",
                height: "100%",
                background: styles.bar_fill_color.clone(),
              }
            }
          }
        }
      }
    }
  )
}

#[allow(non_snake_case)]
#[component]
fn CpuGraphsComponent(cpu_hist: CircularQueue<f32>, memory_swap_hist: [CircularQueue<f32>; 2]) -> Element {
  let styles = use_context::<CpuMemoryStyles>();
  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: styles.graph_h_gap.to_string(),
      rect {
        width: "flex(1)",
        height: styles.graph_height.to_string(),
        border: styles.graph_cpu_border,
        Graph {
          datasets: [cpu_hist],
          graph_colors: [*styles.graph_cpu_fill_color],
        }
      }
      rect {
        width: "flex(1)",
        height: styles.graph_height.to_string(),
        border: styles.graph_memory_border,
        Graph {
          datasets: memory_swap_hist,
          graph_colors: [*styles.graph_memory_fill_color, *styles.graph_swap_fill_color],
        }
      }
    }
  )
}

#[allow(non_snake_case)]
#[component]
fn ProcessTableRow(cmd: String, pid: String, cpu: String, memory: String, color: String, align: String) -> Element {
  let styles = use_context::<CpuMemoryStyles>();
  rsx!(
    rect {
      direction: "horizontal",
      content: "flex",
      color,
      label { width: "flex(1)", text_overflow: "…", "{cmd}" },
      label { width: styles.ps_pid_width.to_string(), text_align: align.clone(), "{pid}" },
      label { width: styles.ps_cpu_width.to_string(), text_align: align.clone(), "{cpu}" },
      label { width: styles.ps_memory_width.to_string(), text_align: align.clone(), "{memory}" },
    }
  )
}

#[allow(non_snake_case)]
#[component]
fn ProcessTableComponent(
  process_data: ReadOnlySignal<ProcessData>,
  num_processes: usize,
  num_cpus: usize,
  top_command: Vec<String>,
) -> Element {
  let styles = use_context::<CpuMemoryStyles>();

  let mut processes = process_data().processes.clone();
  processes.sort_by(|process_1, process_2| process_2.cpu_usage.partial_cmp(&process_1.cpu_usage).unwrap());
  let process_by_cpu = processes[..num_processes.min(processes.len())].to_vec();
  processes.sort_by(|process_1, process_2| process_2.memory_usage.partial_cmp(&process_1.memory_usage).unwrap());
  let process_by_memory = processes[..num_processes.min(processes.len())].to_vec();

  let format_cpu = |process: &ProcessProps| format!("{:.2}", process.cpu_usage / num_cpus as f32);
  let format_memory = |process: &ProcessProps| format_size(process.memory_usage, MEMORY_DECIMAL_PLACES);

  rsx!(
    CursorArea {
      icon: CursorIcon::Pointer,
      rect {
        width: "100%",
        direction: "vertical",
        onclick: move |_| {
          if top_command.is_empty() {
            return;
          }
          let top_command: Vec<String> = top_command
            .iter()
            .map(|part| utils::substitute_env_vars(part))
            .collect();
          let (binary, args) = top_command.split_at(1);
          Command::new(&binary[0]).args(args).status().unwrap();
        },
        ProcessTableRow {
          cmd: "Command", pid: "PID", cpu: "CPU%", memory: "MEM",
          color: styles.ps_header_color.clone(), align: "right",
        },
        ProcessTableRow {
          cmd: "", pid: "", cpu: "🞃", memory: "",
          color: styles.ps_sort_cpu_color.clone(), align: "center",
        },
        for process in process_by_cpu.iter() {
          ProcessTableRow {
            cmd: process.cmd.clone(), pid: process.pid.to_string(),
            cpu: format_cpu(process), memory: format_memory(process),
            color: styles.ps_cpu_color.clone(), align: "right",
          }
        }
        ProcessTableRow {
          cmd: "", pid: "", cpu: "", memory: "🞃",
          color: styles.ps_sort_memory_color.clone(), align: "center",
        },
        for process in process_by_memory.iter() {
          ProcessTableRow {
            cmd: process.cmd.clone(), pid: process.pid.to_string(),
            cpu: format_cpu(process), memory: format_memory(process),
            color: styles.ps_memory_color.clone(), align: "right",
          }
        }
      }
    }
  )
}

#[allow(non_snake_case)]
#[component]
pub fn CpuMemoryComponent() -> Element {
  let config = use_context::<CpuMemoryConfig>();
  let styles = use_context::<CpuMemoryStyles>();
  let global_styles = use_context::<GlobalStyles>();
  let mut clipboard = Clipboard::new().unwrap();

  let refresh_kind = RefreshKind::nothing()
    .with_cpu(CpuRefreshKind::nothing())
    .with_memory(MemoryRefreshKind::nothing().with_ram().with_swap());
  let mut system = System::new_with_specifics(refresh_kind);
  let mut components = Components::new();

  let cpus = system.cpus();
  let mut cpu_model = cpus[0].brand().to_string();
  for &s in CPU_MODEL_REMOVE {
    cpu_model = cpu_model.replace(s, "");
  }
  let num_cpus = cpus.len();

  lazy_static! {
    static ref RE_FREQUENCY: Regex = Regex::new(r"\d+ MHz").unwrap();
  }
  let lshw_output = std::fs::read_to_string("/run/lshw-memory.txt").unwrap();
  let memory_frequency = RE_FREQUENCY.find(&lshw_output).unwrap().as_str();
  let memory_total = system.total_memory();
  let swap_total = system.total_swap();

  let mut cpu_data = use_signal(CpuData::default);
  let mut memory_data = use_signal(MemoryData::default);
  let mut processes_data = use_signal(ProcessData::default);

  let hist_size = ((global_styles.container_width - styles.graph_h_gap) / 2.) as usize;
  let mut cpu_hist = use_signal(|| CircularQueue::with_capacity(hist_size));
  let mut memory_hist = use_signal(|| CircularQueue::with_capacity(hist_size));
  let mut swap_hist = use_signal(|| CircularQueue::with_capacity(hist_size));

  let mut uptime = use_signal(|| 0u64);

  use_hook(move || {
    spawn(async move {
      loop {
        cpu_data.set(get_cpu_data(&mut system, &mut components));
        memory_data.set(get_memory_data(&mut system));

        cpu_hist.write().push(cpu_data().usage / 100.0);
        let memory_ratio = memory_data().memory_usage as f32 / memory_total as f32;
        memory_hist.write().push(memory_ratio);
        let swap_ratio = memory_data().swap_usage as f32 / swap_total as f32;
        swap_hist.write().push(swap_ratio);

        uptime.set(System::uptime());
        processes_data.set(get_process_data(&mut system));

        tokio::time::sleep(std::time::Duration::from_secs(config.update_interval)).await;
      }
    })
  });

  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: global_styles.h_gap.to_string(),
      label { "CPU" },
      CursorArea {
        icon: CursorIcon::Copy,
        label {
          color: styles.value_color.clone(),
          onclick: move |_| { clipboard.set_text(cpu_model.clone()).unwrap(); },
          "{cpu_model}"
        },
      }
      LabelRight { color: styles.value_color.clone(), "{cpu_data().temperature}°C" },
    }
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: global_styles.h_gap.to_string(),
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { "Frequency" },
        LabelRight { color: styles.value_color.clone(), "{cpu_data().frequency:.2} GHz" },
      }
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { "Usage" },
        LabelRight { color: styles.value_color.clone(), "{cpu_data().usage:.1}%" },
      }
    }
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: global_styles.h_gap.to_string(),
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { "Uptime" },
        LabelRight { color: styles.value_color.clone(), "{utils::format_duration(uptime())}" },
      }
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { "Processes" },
        LabelRight {
          color: styles.value_color.clone(),
          "{processes_data().num_running} / {processes_data().processes.len(): >4}",
        },
      }
    }
    CpuBarsComponent { cpu_core_usage: cpu_data().core_usage }
    rect {
      width: "100%",
      direction: "horizontal",
      main_align: "space-between",
      label { "Memory" },
      label { color: styles.value_color.clone(), "{memory_frequency: >8}" },
      label { color: styles.value_color.clone(), "{format_used(memory_data().memory_usage, memory_total): >28}" },
    }
    rect {
      width: "100%",
      direction: "horizontal",
      label { "Swap" },
      LabelRight { color: styles.value_color.clone(), "{format_used(memory_data().swap_usage, swap_total)}" },
    }
    CpuGraphsComponent {
      cpu_hist: cpu_hist(),
      memory_swap_hist: [memory_hist(), swap_hist()],
    }
    ProcessTableComponent {
      process_data: processes_data(),
      num_processes: config.process_list.num_processes,
      num_cpus: num_cpus,
      top_command: config.process_list.top_command.to_vec(),
    }
  )
}
