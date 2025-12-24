use std::collections::HashSet;
use std::ops::Range;
use std::process::Command;
use std::time::Duration;

use async_io::Timer;
use binary_heap_plus::BinaryHeap;
use circular_queue::CircularQueue;
use freya::prelude::*;
use freya::text_edit::Clipboard;
use futures_lite::StreamExt;
use lazy_static::lazy_static;
use velcro::vec;

use regex::Regex;
use sysinfo::{
  Components, CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate,
  RefreshKind, System, UpdateKind,
};

use crate::config::CpuMemoryConfig;
use crate::custom_components::create_graph;
use crate::format_size::format_size;
use crate::freya_utils::{
  border_fill_width, cursor_area, flex_cont_factory, label_with_value_factory, right_value_label, value_label_factory,
};
use crate::serde_structs::SerdeCommand;
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
struct ProcessesData {
  top_cpu: Vec<ProcessProps>,
  top_memory: Vec<ProcessProps>,
  num_total: usize,
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
    .find(|component| component.label().eq("coretemp Package id 0"))
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

fn get_process_data(system: &mut System, num_top_processes: usize) -> ProcessesData {
  let mut res = ProcessesData::default();
  system.refresh_processes_specifics(
    ProcessesToUpdate::All,
    true,
    ProcessRefreshKind::nothing()
      .with_memory()
      .with_cpu()
      .with_cmd(UpdateKind::Always),
  );
  let pid_to_process = system.processes();
  let mut task_pids: HashSet<Pid> = HashSet::new();
  for process in pid_to_process.values() {
    if let Some(tasks) = process.tasks() {
      task_pids.extend(tasks)
    }
  }
  let capacity = num_top_processes + 1;
  let mut top_cpu = BinaryHeap::with_capacity_by(capacity, |p1: &ProcessProps, p2: &ProcessProps| {
    p2.cpu_usage.partial_cmp(&p1.cpu_usage).unwrap()
  });
  let mut top_memory = BinaryHeap::with_capacity_by(capacity, |p1: &ProcessProps, p2: &ProcessProps| {
    p2.memory_usage.partial_cmp(&p1.memory_usage).unwrap()
  });
  for (pid, process) in pid_to_process {
    if task_pids.contains(pid) {
      continue;
    }
    if process.status() == ProcessStatus::Run {
      res.num_running += 1;
    }
    if let Some(live_process) = system.process(*pid) {
      res.num_total += 1;
      let args = live_process
        .cmd()
        .iter()
        .skip(1)
        .fold(String::new(), |res, cur| res + cur.to_str().unwrap() + " ");
      let process = ProcessProps {
        cmd: format!("{} {}", live_process.name().to_str().unwrap(), args),
        pid: *pid,
        cpu_usage: live_process.cpu_usage(),
        memory_usage: live_process.memory(),
      };
      top_cpu.push(process.clone());
      top_memory.push(process.clone());
      if top_cpu.len() > num_top_processes {
        top_cpu.pop();
        top_memory.pop();
      }
    }
  }
  res.top_cpu = top_cpu.into_sorted_vec();
  res.top_memory = top_memory.into_sorted_vec();
  res
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)", "!"];

fn cpu_bars_component(performant_range: Range<usize>, cpu_core_usage: &[f32]) -> Rect {
  let styles = use_consume::<CpuMemoryStyles>();
  let global_styles = use_consume::<GlobalStyles>();
  let flex_cont = flex_cont_factory(styles.bar_h_gap);
  let bar_width =
    (global_styles.container_width - (styles.bars_per_row - 1) as f32 * styles.bar_h_gap) / styles.bars_per_row as f32;
  rect().spacing(styles.bars_v_gap).children(
    (0..cpu_core_usage.len().div_ceil(styles.bars_per_row))
      .map(|i| {
        flex_cont(
          (0..(cpu_core_usage.len() - i * styles.bars_per_row).min(styles.bars_per_row))
            .map(|j| {
              rect()
                .width(Size::px(bar_width))
                .height(Size::px(styles.bar_height))
                .border(Some(if performant_range.contains(&(i * styles.bars_per_row + j)) {
                  border_fill_width(*styles.bar_border_color, styles.bar_border_width)
                } else {
                  border_fill_width(*styles.bar_efficient_border_color, styles.bar_efficient_border_width)
                }))
                .child::<Rect>(
                  rect()
                    .width(Size::percent(cpu_core_usage[i * styles.bars_per_row + j]))
                    .height(Size::percent(100.))
                    .background(if performant_range.contains(&(i * styles.bars_per_row + j)) {
                      *styles.bar_fill_color
                    } else {
                      *styles.bar_efficient_fill_color
                    }),
                )
                .into()
            })
            .collect::<Vec<Element>>(),
        )
        .into()
      })
      .collect::<Vec<Element>>(),
  )
}

fn cpu_graphs_component(cpu_hist: CircularQueue<f32>, memory_swap_hist: [CircularQueue<f32>; 2]) -> Rect {
  let styles = use_consume::<CpuMemoryStyles>();
  let flex_cont = flex_cont_factory(styles.graph_h_gap);
  flex_cont(vec![
    rect()
      .width(Size::flex(1.))
      .height(Size::px(styles.graph_height))
      .border(border_fill_width(
        *styles.graph_cpu_border_color,
        styles.graph_cpu_border_width,
      ))
      .child(create_graph([cpu_hist], [(*styles.graph_cpu_fill_color).into()]))
      .into(),
    rect()
      .width(Size::flex(1.))
      .height(Size::px(styles.graph_height))
      .border(border_fill_width(
        *styles.graph_memory_border_color,
        styles.graph_memory_border_width,
      ))
      .child(create_graph(
        memory_swap_hist,
        [
          (*styles.graph_memory_fill_color).into(),
          (*styles.graph_swap_fill_color).into(),
        ],
      ))
      .into(),
  ])
}

fn process_table_row(
  cmd: &str,
  pid: &str,
  cpu: &str,
  memory: &str,
  color: impl Into<Color>,
  align: TextAlign,
  widths: [f32; 3],
) -> Rect {
  let value_label = |width: f32, text: &str| label().width(Size::px(width)).text_align(align).text(text.to_string());
  rect()
    .direction(Direction::Horizontal)
    .content(Content::Flex)
    .color(color)
    .children([
      label()
        .width(Size::flex(1.))
        .text_overflow(TextOverflow::Custom("…".to_string()))
        .text(cmd.to_string())
        .into(),
      value_label(widths[0], pid).into(),
      value_label(widths[1], cpu).into(),
      value_label(widths[2], memory).into(),
    ])
}

fn process_table_component(processes: ProcessesData, num_cpus: usize, top_command: SerdeCommand) -> CursorArea {
  let styles = use_consume::<CpuMemoryStyles>();

  let format_cpu = |process: &ProcessProps| format!("{:.2}", process.cpu_usage / num_cpus as f32);
  let format_memory = |process: &ProcessProps| format_size(process.memory_usage, MEMORY_DECIMAL_PLACES);
  let header_color = styles.ps_header_color;
  let sort_cpu_color = styles.ps_sort_cpu_color;
  let sort_memory_color = styles.ps_sort_memory_color;
  let widths = [styles.ps_pid_width, styles.ps_cpu_width, styles.ps_memory_width];
  let create_data_row = |p: &ProcessProps, is_cpu: bool| {
    process_table_row(
      &p.cmd,
      &p.pid.to_string(),
      &format_cpu(p),
      &format_memory(p),
      if is_cpu {
        *styles.ps_cpu_color
      } else {
        *styles.ps_memory_color
      },
      TextAlign::Right,
      widths,
    )
  };

  cursor_area(CursorIcon::Pointer).child(
    rect()
      .width(Size::percent(100.))
      .on_pointer_press(move |_| {
        if top_command.is_empty() {
          return;
        }
        let top_command: Vec<String> = top_command
          .iter()
          .map(|part| utils::substitute_env_vars(part))
          .collect();
        let (binary, args) = top_command.split_at(1);
        Command::new(&binary[0]).args(args).status().unwrap();
      })
      .children(vec![
        process_table_row("Command", "PID", "CPU%", "MEM", *header_color, TextAlign::Right, widths).into(),
        process_table_row("", "", "🞃", "", *sort_cpu_color, TextAlign::Right, widths).into(),
        ..processes.top_cpu.iter().map(|p| create_data_row(p, true).into()),
        process_table_row("", "", "", "🞃", *sort_memory_color, TextAlign::Right, widths).into(),
        ..processes.top_memory.iter().map(|p| create_data_row(p, false).into()),
      ]),
  )
}

pub fn cpu_memory_component() -> Rect {
  let config = use_consume::<CpuMemoryConfig>();
  let styles = use_consume::<CpuMemoryStyles>();
  let global_styles = use_consume::<GlobalStyles>();

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
    static ref RE_CPU_RANGE: Regex = Regex::new(r"(\d+)-(\d+)").unwrap();
  }
  let mut cpu_performant_range: Range<usize> = 0..num_cpus;
  if let Ok(contents) = std::fs::read_to_string("/sys/devices/cpu_core/cpus") {
    let groups = RE_CPU_RANGE.captures(&contents).unwrap();
    cpu_performant_range = groups[1].parse().unwrap()..groups[2].parse::<usize>().unwrap() + 1;
  }

  lazy_static! {
    static ref RE_FREQUENCY: Regex = Regex::new(r"\d+ MHz").unwrap();
  }
  let lshw_output = std::fs::read_to_string("/run/lshw-memory.txt").unwrap();
  let memory_frequency = RE_FREQUENCY.find(&lshw_output).unwrap().as_str();
  let memory_total = system.total_memory();
  let swap_total = system.total_swap();

  let mut cpu_data = use_state(CpuData::default);
  let mut memory_data = use_state(MemoryData::default);
  let mut processes_data = use_state(ProcessesData::default);

  let hist_size = ((global_styles.container_width - styles.graph_h_gap) / 2.) as usize;
  let mut cpu_hist = use_state(|| CircularQueue::with_capacity(hist_size));
  let mut memory_hist = use_state(|| CircularQueue::with_capacity(hist_size));
  let mut swap_hist = use_state(|| CircularQueue::with_capacity(hist_size));

  let mut uptime = use_state(|| 0u64);

  use_hook(|| {
    spawn(async move {
      loop {
        cpu_data.set(get_cpu_data(&mut system, &mut components));
        memory_data.set(get_memory_data(&mut system));

        cpu_hist.write().push(cpu_data.read().usage / 100.0);
        let memory_ratio = memory_data.read().memory_usage as f32 / memory_total as f32;
        memory_hist.write().push(memory_ratio);
        let swap_ratio = memory_data.read().swap_usage as f32 / swap_total as f32;
        swap_hist.write().push(swap_ratio);

        uptime.set(System::uptime());
        processes_data.set(get_process_data(&mut system, config.process_list.num_processes));

        Timer::interval(Duration::from_secs(config.update_interval))
          .next()
          .await;
      }
    })
  });

  let value_color: Color = (*styles.value_color).into();
  let flex_cont = flex_cont_factory(global_styles.h_gap);
  let value_label = value_label_factory(value_color);
  let label_with_value = label_with_value_factory(Color::default(), value_color);

  rect().children([
    flex_cont(vec![
      label().text("CPU").into(),
      cursor_area(CursorIcon::Copy)
        .child(value_label(cpu_model.clone()).on_mouse_down(move |_| Clipboard::set(cpu_model.clone()).unwrap()))
        .into(),
      right_value_label(value_color, format!("{}°C", cpu_data.read().temperature)).into(),
    ])
    .into(),
    flex_cont(vec![
      label_with_value("Frequency", format!("{:.2} GHz", cpu_data.read().frequency)).into(),
      label_with_value("Usage", format!("{:.1}%", cpu_data.read().usage)).into(),
    ])
    .into(),
    flex_cont(vec![
      label_with_value("Uptime", utils::format_duration(uptime())).into(),
      label_with_value(
        "Processes",
        format!(
          "{} / {: >4}",
          processes_data.read().num_running,
          processes_data.read().num_total
        ),
      )
      .into(),
    ])
    .into(),
    cpu_bars_component(cpu_performant_range, &cpu_data.read().core_usage).into(),
    rect()
      .width(Size::percent(100.))
      .direction(Direction::Horizontal)
      .main_align(Alignment::SpaceBetween)
      .children([
        label().text("Memory").into(),
        value_label(format!("{: >8}", memory_frequency)).into(),
        value_label(format!(
          "{: >28}",
          format_used(memory_data.read().memory_usage, memory_total)
        ))
        .into(),
      ])
      .into(),
    rect()
      .width(Size::percent(100.))
      .direction(Direction::Horizontal)
      .children([
        label().text("Swap").into(),
        right_value_label(value_color, format_used(memory_data.read().swap_usage, swap_total)).into(),
      ])
      .into(),
    cpu_graphs_component(
      (*cpu_hist.read()).clone(),
      [(*memory_hist.read()).clone(), (*swap_hist.read()).clone()],
    )
    .into(),
    process_table_component(
      (*processes_data.read()).clone(),
      num_cpus,
      config.process_list.top_command,
    )
    .into(),
  ])
}
