mod cpu_memory_widget;
mod machine_info_widget;
mod weather_widget;
mod window;

pub use cpu_memory_widget::{CpuMemoryWidget, CpuMemoryWidgetUpdater};
pub use machine_info_widget::{update_machine_info_widget, MachineInfoWidget};
pub use weather_widget::{WeatherWidget, WeatherWidgetUpdater};
pub use window::Window;
