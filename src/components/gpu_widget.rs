use gtk::{glib, Builder};
use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor, TemperatureThreshold};
use nvml_wrapper::{Device, Nvml};
use std::sync::Arc;

use crate::config::GpuProps;
use crate::format_size::format_size;
use crate::gtk_utils::{set_copyable_label, set_label};

#[derive(Clone)]
pub struct GpuWidget {
  nvml: Arc<Nvml>,
  builder: Arc<Builder>,
}

const MEMORY_DECIMAL_PLACES: usize = 1usize;

impl GpuWidget {
  pub fn build(props: GpuProps) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/gpu_widget.ui");
    let container: gtk::Box = builder.object("gpu_widget").unwrap();

    let props = Arc::new(props);
    let nvml = Nvml::init().unwrap();

    let mut updater = GpuWidget {
      builder: Arc::new(builder),
      nvml: Arc::new(nvml),
    };
    updater.update_static_props();
    updater.update(props);
    container
  }

  fn get_gpu(nvml: &Nvml) -> Device {
    // We can't use `self` as an argument here, as the returned device references `self.nvml`
    nvml.device_by_index(0).unwrap()
  }

  fn update_static_props(&self) {
    let gpu = GpuWidget::get_gpu(&self.nvml);
    let gpu_model = gpu.name().unwrap();
    set_copyable_label(&self.builder, "gpu_model", gpu_model);
  }

  fn update_gpu(&self) {
    let gpu = GpuWidget::get_gpu(&self.nvml);

    let gpu_temperature = gpu.temperature(TemperatureSensor::Gpu).unwrap();
    let gpu_threshold = gpu.temperature_threshold(TemperatureThreshold::Shutdown).unwrap();
    let temperature_string = format!("{:.0}°C/{:.0}°C", gpu_temperature, gpu_threshold);
    set_label(&self.builder, "gpu_temperature", &temperature_string);

    let utilization_rates = gpu.utilization_rates().unwrap();
    set_label(&self.builder, "gpu_usage", &format!("{}%", utilization_rates.gpu));

    let gpu_frequency = gpu.clock(Clock::Graphics, ClockId::Current).unwrap();
    set_label(&self.builder, "gpu_frequency", &format!("{} MHz", gpu_frequency));

    let memory_frequency = gpu.clock(Clock::Memory, ClockId::Current).unwrap();
    let memory_frequency_str = format!("{} MHz", memory_frequency);
    set_label(&self.builder, "gpu_memory_frequency", &memory_frequency_str);

    let memory_info = gpu.memory_info().unwrap();
    let memory_usage = format!(
      "{}/{} = {:>3.0}%",
      &format_size(memory_info.used, MEMORY_DECIMAL_PLACES),
      &format_size(memory_info.total, MEMORY_DECIMAL_PLACES),
      memory_info.used as f32 / memory_info.total as f32 * 100.0
    );
    set_label(&self.builder, "gpu_memory_usage", &memory_usage);
  }

  fn update(&mut self, props: Arc<GpuProps>) {
    self.update_gpu();

    let mut self_clone = self.clone();
    glib::source::timeout_add_seconds_local_once(props.update_interval, move || self_clone.update(props));
  }
}
