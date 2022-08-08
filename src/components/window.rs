use gtk::prelude::{BoxExt, StyleContextExt, WidgetExt};
use gtk::{Builder, Orientation, Separator, Window};

use crate::components::disk_widget::DiskWidget;
use crate::components::gpu_widget::GpuWidget;
use crate::components::{CpuMemoryWidget, MachineInfoWidget, WeatherWidget};
use crate::ConfigProps;

pub fn build_window(config_props: &ConfigProps) -> Window {
  let builder = Builder::from_resource("/org/dogky/window.ui");
  let window = builder.object::<Window>("dogky_window").unwrap();
  let window_padding = window.style_context().padding();
  let container_width = config_props.width - (window_padding.left() + window_padding.right()) as u32;
  let container: gtk::Box = builder.object("container").unwrap();

  let weather_widget = WeatherWidget::build(config_props.weather.clone());
  container.append(&weather_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let machine_info_widget = MachineInfoWidget::build();
  container.append(&machine_info_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let cpu_memory_widget = CpuMemoryWidget::build(config_props.cpu_memory.clone(), container_width);
  container.append(&cpu_memory_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let disk_widget = DiskWidget::build(config_props.disk.clone(), container_width);
  container.append(&disk_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let gpu_widget = GpuWidget::build(config_props.gpu.clone());
  container.append(&gpu_widget);

  window
}
