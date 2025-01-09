use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{column, mouse_area, row, text};
use iced::{clipboard, time, Element, Length, Subscription, Task};
use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor, TemperatureThreshold};
use nvml_wrapper::{Device, Nvml};
use std::time::Duration;

use crate::config::GpuProps;
use crate::format_size::format_size;
use crate::message::Message;
use crate::styles::gpu as styles;
use crate::ui_utils::{expand_right, space_row, WithStyle};

pub struct GpuComponent {
  config_props: GpuProps,
  nvml: Nvml,
  model: String,
  live: GpuLiveProps,
}

#[derive(Default)]
struct GpuLiveProps {
  temperature: u32,
  temperature_threshold: u32,
  utilization_rates: u32,
  gpu_frequency: u32,
  memory_frequency: u32,
  memory_used: u64,
  memory_total: u64,
}

const MEMORY_DECIMAL_PLACES: usize = 1usize;

impl GpuComponent {
  pub fn new(config_props: GpuProps) -> Self {
    let nvml = Nvml::init().unwrap();

    let gpu = GpuComponent::get_gpu(&nvml);
    let model = gpu.name().unwrap();

    Self {
      config_props,
      nvml,
      model,
      live: GpuLiveProps::default(),
    }
  }

  fn get_gpu(nvml: &Nvml) -> Device {
    // We can't use `self` as an argument here, as the returned device references `self.nvml`
    nvml.device_by_index(0).unwrap()
  }

  fn update_data(&mut self) {
    let live = &mut self.live;

    let gpu = GpuComponent::get_gpu(&self.nvml);

    live.temperature = gpu.temperature(TemperatureSensor::Gpu).unwrap();
    live.temperature_threshold = gpu.temperature_threshold(TemperatureThreshold::Shutdown).unwrap();
    live.utilization_rates = gpu.utilization_rates().unwrap().gpu;
    live.gpu_frequency = gpu.clock(Clock::Graphics, ClockId::Current).unwrap();
    live.memory_frequency = gpu.clock(Clock::Memory, ClockId::Current).unwrap();

    let memory_info = gpu.memory_info().unwrap();
    live.memory_used = memory_info.used;
    live.memory_total = memory_info.total;
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::GPUTick => {
        self.update_data();
        Task::none()
      }
      Message::GPUModelClick => clipboard::write(self.model.to_string()),
      _ => Task::none(),
    }
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let props = &self.config_props;
    time::every(Duration::from_secs(props.update_interval)).map(|_instant| Message::GPUTick)
  }

  pub fn view(&self) -> Element<Message> {
    let name_style = WithStyle::new(styles::NAME_COLOR);
    let usage_name_style = WithStyle::new(styles::USAGE_NAME_COLOR);
    let value_style = WithStyle::new(styles::VALUE_COLOR);

    let live = &self.live;

    let model_text = name_style.text(self.model.to_string());
    let model_copy = mouse_area(model_text)
      .interaction(Interaction::Copy)
      .on_press(Message::GPUModelClick);

    let temperature_string = format!("{:.0}°C/{:.0}°C", live.temperature, live.temperature_threshold);

    let memory_frequency_str = format!("{} MHz", live.memory_frequency);
    let memory_usage_str = format!(
      "{}/{} = {:>3.0}%",
      &format_size(live.memory_used, MEMORY_DECIMAL_PLACES),
      &format_size(live.memory_total, MEMORY_DECIMAL_PLACES),
      live.memory_used as f32 / live.memory_total as f32 * 100.0
    );

    column![
      row![
        space_row![row![text("GPU"), model_copy]],
        expand_right![value_style.text(temperature_string)]
      ],
      space_row![row![
        row![
          usage_name_style.text("Usage"),
          expand_right![value_style.text(format!("{}%", live.utilization_rates))]
        ]
        .width(Length::Fill),
        row![
          usage_name_style.text("Frequency"),
          expand_right![value_style.text(format!("{} MHz", live.gpu_frequency))]
        ]
        .width(Length::Fill),
      ]
      .width(Length::Fill)],
      row![
        usage_name_style.text("Memory").width(Length::Fill),
        value_style.text(memory_frequency_str).width(Length::Fill),
        value_style.text(memory_usage_str),
      ]
      .width(Length::Fill),
    ]
    .width(Length::Fill)
    .into()
  }
}
