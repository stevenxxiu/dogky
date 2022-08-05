use gtk::prelude::BoxExt;
use gtk::{Builder, Orientation, Separator, Window};

use crate::components::{CpuMemoryWidget, MachineInfoWidget, WeatherWidget};
use crate::ConfigProps;

pub fn build_window(config_props: &ConfigProps) -> Window {
  let builder = Builder::from_resource("/org/dogky/window.ui");
  let container: gtk::Box = builder.object("container").unwrap();

  let weather_widget = WeatherWidget::build(config_props.weather.clone());
  container.append(&weather_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let machine_info_widget = MachineInfoWidget::build();
  container.append(&machine_info_widget);

  container.append(&Separator::new(Orientation::Horizontal));
  let cpu_memory_widget = CpuMemoryWidget::build(config_props.cpu_memory.clone());
  container.append(&cpu_memory_widget);

  builder.object::<Window>("dogky_window").unwrap()
}
