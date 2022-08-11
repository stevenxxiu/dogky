use gtk::{glib, Builder};
use nvml_wrapper::enum_wrappers::device::{Clock, ClockId, TemperatureSensor, TemperatureThreshold};
use nvml_wrapper::{Device, Nvml};
use std::sync::Arc;

use crate::config::GpuProps;
use crate::format_size::format_size;
use crate::gtk_utils::{set_copyable_label, set_label};

pub struct GpuWidget {
  builder: Arc<Builder>,
  nvml: Arc<Nvml>,
}

const MEMORY_DECIMAL_PLACES: usize = 1usize;

impl GpuWidget {
  pub fn build(props: GpuProps) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/gpu_widget.ui");
    let container: gtk::Box = builder.object("gpu_widget").unwrap();

    let props = Arc::new(props);
    let nvml = Nvml::init().unwrap();
    let gpu = GpuWidget::get_gpu(&nvml);
    GpuWidget::update_static_props(&gpu, &builder);

    let updater = GpuWidget {
      builder: Arc::new(builder),
      nvml: Arc::new(nvml),
    };
    updater.update(props);
    container
  }

  fn get_gpu(nvml: &Nvml) -> Device {
    nvml.device_by_index(0).unwrap()
  }

  fn update_static_props(gpu: &Device, builder: &Builder) {
    let gpu_model = gpu.name().unwrap();
    set_copyable_label(builder, "gpu_model", gpu_model);
  }

  fn update_gpu(gpu: &Device, builder: &Builder) {
    let gpu_temperature = gpu.temperature(TemperatureSensor::Gpu).unwrap();
    let gpu_threshold = gpu.temperature_threshold(TemperatureThreshold::Shutdown).unwrap();
    let temperature_string = format!("{:.0}°C/{:.0}°C", gpu_temperature, gpu_threshold);
    set_label(builder, "gpu_temperature", &temperature_string);

    let utilization_rates = gpu.utilization_rates().unwrap();
    set_label(builder, "gpu_usage", &format!("{}%", utilization_rates.gpu));

    let gpu_frequency = gpu.clock(Clock::Graphics, ClockId::Current).unwrap();
    set_label(builder, "gpu_frequency", &format!("{} MHz", gpu_frequency));

    let memory_frequency = gpu.clock(Clock::Memory, ClockId::Current).unwrap();
    set_label(builder, "gpu_memory_frequency", &format!("{} MHz", memory_frequency));

    let memory_info = gpu.memory_info().unwrap();
    let memory_usage = format!(
      "{}/{} = {:>3.0}%",
      &format_size(memory_info.used, MEMORY_DECIMAL_PLACES),
      &format_size(memory_info.total, MEMORY_DECIMAL_PLACES),
      memory_info.used as f32 / memory_info.total as f32 * 100.0
    );
    set_label(builder, "gpu_memory_usage", &memory_usage);
  }

  fn update(self, props: Arc<GpuProps>) {
    let gpu = GpuWidget::get_gpu(&self.nvml);
    GpuWidget::update_gpu(&gpu, &self.builder);
    glib::source::timeout_add_seconds_local_once(props.update_interval, move || self.update(props));
  }
}
