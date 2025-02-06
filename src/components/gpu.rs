use arboard::Clipboard;
use freya::prelude::*;

use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor, TemperatureThreshold};
use nvml_wrapper::{Device, Nvml};

use crate::config::GpuConfig;
use crate::custom_components::LabelRight;
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

fn get_gpu(nvml: &Nvml) -> Device {
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

#[allow(non_snake_case)]
#[component]
pub fn GpuComponent(nvml_signal: ReadOnlySignal<Nvml>) -> Element {
  let config = use_context::<GpuConfig>();
  let styles = use_context::<GpuStyles>();
  let global_styles = use_context::<GlobalStyles>();

  let mut clipboard = Clipboard::new().unwrap();

  let nvml = nvml_signal.peek();
  let gpu = get_gpu(&nvml);
  let model = gpu.name().unwrap();
  let temperature_threshold = gpu.temperature_threshold(TemperatureThreshold::Shutdown).unwrap();
  let memory_info = gpu.memory_info().unwrap();
  let memory_total = memory_info.total;

  let mut data = use_signal(GpuData::default);

  use_hook(move || {
    spawn(async move {
      loop {
        data.set(get_gpu_data(&nvml_signal.peek()));
        tokio::time::sleep(std::time::Duration::from_secs(config.update_interval)).await;
      }
    })
  });

  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      spacing: global_styles.h_gap.to_string(),
      label { "GPU" },
      CursorArea {
        icon: CursorIcon::Copy,
        label {
          color: styles.name_color.clone(),
          onclick: move |_| { clipboard.set_text(model.clone()).unwrap() },
          "{model}"
        },
      }
      LabelRight { color: styles.value_color.clone(), "{data().temperature:.0}°C/{temperature_threshold:.0}°C" },
    }
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: global_styles.h_gap.to_string(),
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { color: styles.usage_name_color.clone(), "Usage" },
        LabelRight { color: styles.value_color.clone(), "{data().utilization_rates}%" },
      }
      rect {
        width: "flex(1)",
        direction: "horizontal",
        label { color: styles.usage_name_color.clone(), "Frequency" },
        LabelRight { color: styles.value_color.clone(), "{data().gpu_frequency} MHz" },
      }
    }
    rect {
      width: "100%",
      direction: "horizontal",
      main_align: "space-between",
      label { color: styles.usage_name_color.clone(), "Memory" },
      label { color: styles.value_color.clone(), "{data().memory_frequency: >4} MHz" },
      label { color: styles.value_color.clone(), "{format_used(data().memory_used, memory_total)}" },
    }
  )
}
