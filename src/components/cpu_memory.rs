use circular_queue::CircularQueue;
use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{canvas, column, container, mouse_area, row, text, Column, MouseArea, Row};
use iced::{clipboard, time, Color, Element, Length, Subscription, Task};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;
use std::process::Command;
use std::time::Duration;
use sysinfo::{
  Components, CpuRefreshKind, MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessStatus, ProcessesToUpdate,
  RefreshKind, System, UpdateKind,
};

use crate::config::CpuMemoryProps;
use crate::custom_components::{Bar, Graph};
use crate::format_size::format_size;
use crate::message::{CPUMemoryMessage, Message};
use crate::styles::cpu_memory as styles;
use crate::ui_utils::{expand_right, space_row, WithStyle};
use crate::utils;

pub struct CpuMemoryComponent {
  config_props: CpuMemoryProps,
  container_width: f32,
  char_width: f32,
  system: System,
  components: Components,
  cpu_model: String,
  num_cpus: usize,
  memory_frequency: String,
  memory_total: u64,
  swap_total: u64,
  live: CpuMemoryLiveProps,
  history: HistoryProps,
}

#[derive(Default)]
struct CpuMemoryLiveProps {
  cpu_frequency: f32,
  cpu_temperature: f32,
  cpu_usage: f32,
  cpu_core_usage: Vec<f32>,
  uptime: u64,
  processes: Vec<ProcessProps>,
  num_running: usize,
  memory_usage: u64,
  swap_usage: u64,
}

struct HistoryProps {
  cpu: CircularQueue<f32>,
  memory: CircularQueue<f32>,
  swap: CircularQueue<f32>,
}

#[derive(Clone)]
struct ProcessProps {
  cmd: String,
  pid: Pid,
  cpu_usage: f32,
  memory_usage: u64,
}

const CPU_MODEL_REMOVE: &[&str] = &["(R)", "(TM)", "!"];
const MEMORY_DECIMAL_PLACES: usize = 1usize;

impl CpuMemoryComponent {
  pub fn new(config_props: CpuMemoryProps, container_width: f32) -> Self {
    let char_width = crate::styles::get_char_dims().width;

    let refresh_kind = RefreshKind::nothing()
      .with_cpu(CpuRefreshKind::nothing())
      .with_memory(MemoryRefreshKind::nothing().with_ram().with_swap());
    let system = System::new_with_specifics(refresh_kind);
    let components = Components::new();

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
    let memory_frequency = RE_FREQUENCY
      .find(&lshw_output)
      .into_iter()
      .next()
      .unwrap()
      .as_str()
      .to_string();
    let memory_total = system.total_memory();
    let swap_total = system.total_swap();

    let process_data_size = ((container_width - styles::GRAPH_H_GAP) / 2.0) as usize;

    Self {
      config_props,
      char_width,
      container_width,
      system,
      components,
      cpu_model,
      num_cpus,
      memory_frequency,
      memory_total,
      swap_total,
      live: CpuMemoryLiveProps::default(),
      history: HistoryProps {
        cpu: CircularQueue::with_capacity(process_data_size),
        memory: CircularQueue::with_capacity(process_data_size),
        swap: CircularQueue::with_capacity(process_data_size),
      },
    }
  }

  fn update_cpu_data(&mut self) {
    let system = &mut self.system;
    system.refresh_cpu_specifics(CpuRefreshKind::nothing().with_frequency().with_cpu_usage());

    let cpus = system.cpus();
    self.live.cpu_frequency = cpus[0].frequency() as f32 / 1000.0;

    self.components.refresh(true);
    self.live.cpu_temperature = self
      .components
      .into_iter()
      .find(|component| component.label().eq("Package id 0"))
      .unwrap()
      .temperature()
      .unwrap();

    self.live.cpu_usage = system.global_cpu_usage();
    self.live.cpu_core_usage = cpus.iter().map(|cpu| cpu.cpu_usage()).collect();
    self.history.cpu.push(self.live.cpu_usage / 100.0);
  }

  fn update_memory(&mut self) {
    self.system.refresh_memory();
    self.live.memory_usage = self.system.used_memory();
    self.live.swap_usage = self.system.used_swap();
    self
      .history
      .memory
      .push(self.live.memory_usage as f32 / self.memory_total as f32);
    self
      .history
      .swap
      .push(self.live.swap_usage as f32 / self.swap_total as f32);
  }

  fn update_system(&mut self) {
    self.live.uptime = System::uptime();
  }

  fn update_processes(&mut self) {
    // Create another `System` from scratch every time, or we end up with more processes than there should be
    let mut system_new = System::new_with_specifics(RefreshKind::nothing());
    system_new.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    self.system.refresh_processes_specifics(
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
    self.live.processes.clear();
    self.live.num_running = 0;
    for (pid, process) in pid_to_process {
      if task_pids.contains(pid) {
        continue;
      }
      if process.status() == ProcessStatus::Run {
        self.live.num_running += 1;
      }
      let args = process
        .cmd()
        .iter()
        .skip(1)
        .fold(String::new(), |res, cur| res + cur.to_str().unwrap() + " ");
      if let Some(live_process) = self.system.process(*pid) {
        self.live.processes.push(ProcessProps {
          cmd: format!("{} {}", live_process.name().to_str().unwrap(), args),
          pid: *pid,
          cpu_usage: live_process.cpu_usage(),
          memory_usage: live_process.memory(),
        });
      }
    }
  }

  fn update_data(&mut self) {
    self.update_cpu_data();
    self.update_memory();
    self.update_system();
    self.update_processes();
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    if let Message::CPUMemory(message) = message {
      return match message {
        CPUMemoryMessage::Tick => {
          self.update_data();
          Task::none()
        }
        CPUMemoryMessage::CPUModelClick => clipboard::write(self.cpu_model.to_string()),
        CPUMemoryMessage::ProcessTableClick => {
          let top_command = &self.config_props.process_list.top_command;
          if top_command.is_empty() {
            return Task::none();
          }
          let top_command: Vec<String> = top_command
            .iter()
            .map(|part| utils::substitute_env_vars(part))
            .collect();
          let (binary, args) = top_command.split_at(1);
          Command::new(&binary[0]).args(args).status().unwrap();
          Task::none()
        }
      };
    }
    Task::none()
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let props = &self.config_props;
    time::every(Duration::from_secs(props.update_interval)).map(|_instant| Message::CPUMemory(CPUMemoryMessage::Tick))
  }

  fn view_cpu_bars(&self) -> Element<'_, Message> {
    let props = &self.config_props.cpu_bars;
    let n = props.num_per_row;
    let bar_width: f32 = (self.container_width - (n - 1) as f32 * styles::BAR_H_GAP) / n as f32;
    column(self.live.cpu_core_usage.chunks(n).map(|row_cpu_usages| {
      row(row_cpu_usages.iter().map(|&cpu_usage| {
        let cpu_usage = cpu_usage / 100.0;
        let bar = Bar {
          value: cpu_usage,
          width: bar_width,
          height: styles::BAR_HEIGHT,
          fill_color: styles::BAR_FILL_COLOR,
          border_color: styles::BAR_BORDER_COLOR,
          ..Default::default()
        };
        container(canvas(bar).width(bar_width).height(styles::BAR_HEIGHT)).into()
      }))
      .width(Length::Fill)
      .spacing(styles::BAR_H_GAP)
      .into()
    }))
    .spacing(styles::BARS_V_GAP)
    .width(Length::Fill)
    .into()
  }

  fn view_graphs(&self) -> Row<Message> {
    let graph_width: f32 = (self.container_width - styles::GRAPH_H_GAP) / 2.0;
    row![
      container(
        canvas(Graph {
          datasets: vec![self.history.cpu.clone()],
          width: graph_width,
          height: styles::GRAPH_HEIGHT,
          border_color: styles::GRAPH_CPU_BORDER_COLOR,
          graph_colors: vec![styles::GRAPH_CPU_FILL_COLOR],
          cache: canvas::Cache::new(),
        })
        .width(graph_width)
        .height(styles::GRAPH_HEIGHT)
      ),
      container(
        canvas(Graph {
          datasets: vec![self.history.memory.clone(), self.history.swap.clone()],
          width: graph_width,
          height: styles::GRAPH_HEIGHT,
          border_color: styles::GRAPH_MEMORY_BORDER_COLOR,
          graph_colors: vec![styles::GRAPH_MEMORY_FILL_COLOR, styles::GRAPH_SWAP_FILL_COLOR],
          cache: canvas::Cache::new(),
        })
        .width(graph_width)
        .height(styles::GRAPH_HEIGHT)
      )
    ]
    .width(Length::Fill)
    .spacing(styles::GRAPH_H_GAP)
  }

  fn view_process_table(&self) -> MouseArea<'_, Message> {
    let live = &self.live;
    let ps_props = &self.config_props.process_list;

    let cmd_cell = |s: String| text(s).width(Length::Fill);
    let pid_cell = |s: String| text(s).width(Length::Fixed(ps_props.pid_width));
    let cpu_cell = |s: String| text(s).width(Length::Fixed(ps_props.cpu_width));
    let memory_cell = |s: String| text(s).width(Length::Fixed(ps_props.memory_width));
    let cmd_width = self.container_width - ps_props.pid_width - ps_props.cpu_width - ps_props.memory_width;
    let max_cmd_chars = (cmd_width / self.char_width) as usize;
    let format_cmd = |s: &str| {
      if s.len() > max_cmd_chars {
        s[..(max_cmd_chars - 1)].to_string() + "…"
      } else {
        s.to_string()
      }
    };

    let process_to_row = |process: &ProcessProps, color: Color| {
      row![
        cmd_cell(format_cmd(&process.cmd).to_string()).color(color),
        pid_cell(process.pid.to_string())
          .align_x(Horizontal::Right)
          .color(color),
        cpu_cell(format!("{:.2}", process.cpu_usage / self.num_cpus as f32))
          .align_x(Horizontal::Right)
          .color(color),
        memory_cell(format_size(process.memory_usage, MEMORY_DECIMAL_PLACES))
          .align_x(Horizontal::Right)
          .color(color),
      ]
    };

    let mut processes = live.processes.clone();

    processes.sort_by(|process_1, process_2| process_2.cpu_usage.partial_cmp(&process_1.cpu_usage).unwrap());
    let process_by_cpu: Column<Message> = Column::with_children(
      processes
        .iter()
        .take(ps_props.num_processes)
        .map(|process| process_to_row(process, styles::PS_CPU_COLOR))
        .map(Element::from),
    );

    processes.sort_by(|process_1, process_2| process_2.memory_usage.partial_cmp(&process_1.memory_usage).unwrap());
    let process_by_memory: Column<Message> = Column::with_children(
      processes
        .iter()
        .take(ps_props.num_processes)
        .map(|process| process_to_row(process, styles::PS_MEMORY_COLOR))
        .map(Element::from),
    );

    let process_table = column![
      row![
        cmd_cell("Command".to_string()).color(styles::PS_HEADER_COLOR),
        pid_cell("PID".to_string())
          .color(styles::PS_HEADER_COLOR)
          .align_x(Horizontal::Right),
        cpu_cell("CPU%".to_string())
          .color(styles::PS_HEADER_COLOR)
          .align_x(Horizontal::Right),
        memory_cell("MEM".to_string())
          .color(styles::PS_HEADER_COLOR)
          .align_x(Horizontal::Right),
      ],
      row![
        cmd_cell("".to_string()),
        pid_cell("".to_string()),
        cpu_cell("🞃".to_string())
          .font(styles::PS_SORT_FONT)
          .color(styles::PS_SORT_CPU_COLOR)
          .align_x(Horizontal::Center),
        memory_cell("".to_string()),
      ],
      process_by_cpu,
      row![
        cmd_cell("".to_string()),
        pid_cell("".to_string()),
        cpu_cell("".to_string()),
        memory_cell("🞃".to_string())
          .font(styles::PS_SORT_FONT)
          .color(styles::PS_SORT_MEMORY_COLOR)
          .align_x(Horizontal::Center),
      ],
      process_by_memory,
    ];
    mouse_area(process_table)
      .interaction(Interaction::Pointer)
      .on_press(Message::CPUMemory(CPUMemoryMessage::ProcessTableClick))
  }

  pub fn view(&self) -> Element<Message> {
    let value_style = WithStyle::new(styles::VALUE_COLOR);

    let live = &self.live;

    let cpu_model_text = value_style.text(self.cpu_model.to_string());
    let cpu_model_copy = mouse_area(cpu_model_text)
      .interaction(Interaction::Copy)
      .on_press(Message::CPUMemory(CPUMemoryMessage::CPUModelClick));

    let processes_status = format!("{} / {: >4}", live.num_running, live.processes.len());

    let format_memory = |used, total| {
      format!(
        "{: >10}/{: >10} = {: >3.0}%",
        format_size(used, MEMORY_DECIMAL_PLACES),
        format_size(total, MEMORY_DECIMAL_PLACES),
        (used as f32) / (total as f32) * 100.0
      )
    };

    column![
      row![
        space_row![row![text("CPU"), cpu_model_copy]],
        expand_right![value_style.text(format!("{}°C", live.cpu_temperature))]
      ],
      space_row![row![
        row![
          text("Frequency"),
          expand_right![value_style.text(format!("{:.2} GHz", live.cpu_frequency))]
        ]
        .width(Length::Fill),
        row![
          text("Usage"),
          expand_right![value_style.text(format!("{:.1}%", live.cpu_usage))]
        ]
        .width(Length::Fill),
      ]
      .width(Length::Fill)],
      space_row![row![
        row![
          text("Uptime"),
          expand_right![value_style.text(utils::format_duration(live.uptime))]
        ]
        .width(Length::Fill),
        row![text("Processes"), expand_right![value_style.text(processes_status)]].width(Length::Fill),
      ]
      .width(Length::Fill)],
      self.view_cpu_bars(),
      row![
        text("Memory").width(Length::Fill),
        value_style.text(self.memory_frequency.to_string()).width(Length::Fill),
        value_style.text(format_memory(live.memory_usage, self.memory_total)),
      ]
      .width(Length::Fill),
      row![
        text("Swap").width(Length::Fill),
        value_style.text(format_memory(live.swap_usage, self.swap_total)),
      ]
      .width(Length::Fill),
      self.view_graphs(),
      self.view_process_table(),
    ]
    .width(Length::Fill)
    .into()
  }
}
