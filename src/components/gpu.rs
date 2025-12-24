use std::sync::Arc;
use std::time::Duration;

use async_io::Timer;
use freya::prelude::*;

use freya::text_edit::Clipboard;
use futures_lite::stream::StreamExt;
use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor, TemperatureThreshold};
use nvml_wrapper::{Device, Nvml};

use crate::config::GpuConfig;
use crate::freya_utils::{
  color_label, cursor_area, flex_cont_factory, horizontal_cont_factory, label_with_value_factory, right_value_label,
  value_label_factory,
};
use crate::styles_config::{GlobalStyles, GpuStyles};
use crate::utils::format_used;

#[derive(Default, Clone, Debug)]
struct GpuData {
  temperature: u32,
  utilization_rates: u32,
  gpu_frequency: u32,
  memory_frequency: u32,
  memory_used: u64,
}

fn get_gpu(nvml: &Nvml) -> Device<'_> {
  nvml.device_by_index(0).unwrap()
}

fn get_gpu_data(nvml: &Nvml) -> GpuData {
  let mut res = GpuData::default();
  let gpu = get_gpu(nvml);

  res.temperature = gpu.temperature(TemperatureSensor::Gpu).unwrap();
  res.utilization_rates = gpu.utilization_rates().unwrap().gpu;
  res.gpu_frequency = gpu.clock(Clock::Graphics, ClockId::Current).unwrap();
  res.memory_frequency = gpu.clock(Clock::Memory, ClockId::Current).unwrap();

  let memory_info = gpu.memory_info().unwrap();
  res.memory_used = memory_info.used;
  res
}

pub struct GpuComponent {
  pub nvml: Arc<Nvml>,
}

impl PartialEq for GpuComponent {
  fn eq(&self, _other: &Self) -> bool {
    true
  }
}

impl Render for GpuComponent {
  fn render(&self) -> impl IntoElement {
    let config = use_consume::<GpuConfig>();
    let styles = use_consume::<GpuStyles>();
    let global_styles = use_consume::<GlobalStyles>();

    let gpu = get_gpu(&self.nvml);
    let model = gpu.name().unwrap();
    let temperature_threshold = gpu.temperature_threshold(TemperatureThreshold::Shutdown).unwrap();
    let memory_info = gpu.memory_info().unwrap();
    let memory_total = memory_info.total;

    let mut data = use_state(GpuData::default);

    let nvml = Arc::clone(&self.nvml);
    use_hook(|| {
      spawn(async move {
        loop {
          data.set(get_gpu_data(&nvml));
          Timer::interval(Duration::from_secs(config.update_interval))
            .next()
            .await;
        }
      })
    });

    let value_color: Color = (*styles.value_color).into();
    let flex_cont = flex_cont_factory(global_styles.h_gap);
    let horizontal_cont = horizontal_cont_factory(global_styles.h_gap);
    let label_with_value = label_with_value_factory((*styles.usage_name_color).into(), value_color);
    let value_label = value_label_factory(value_color);

    rect().children([
      horizontal_cont(vec![
        label().text("GPU").into(),
        cursor_area(CursorIcon::Copy)
          .child(
            color_label(*styles.name_color, model.clone())
              .on_mouse_down(move |_| Clipboard::set(model.clone()).unwrap()),
          )
          .into(),
        right_value_label(
          value_color,
          format!("{:.0}°C/{:.0}°C", data.read().temperature, temperature_threshold),
        )
        .into(),
      ])
      .into(),
      flex_cont(vec![
        label_with_value("Usage", format!("{}%", data.read().utilization_rates)).into(),
        label_with_value("Frequency", format!("{} MHz", data.read().gpu_frequency)).into(),
      ])
      .into(),
      horizontal_cont(vec![
        color_label(*styles.usage_name_color, "Memory").into(),
        value_label(format!("{: >4} MHz", data.read().memory_frequency)).into(),
        value_label(format_used(data.read().memory_used, memory_total)).into(),
      ])
      .main_align(Alignment::SpaceBetween)
      .into(),
    ])
  }
}
