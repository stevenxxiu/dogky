use iced::event::Event;

#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(Event),
  WeatherWidgetTick,
  WeatherWidgetClick,
  MachineInfoKernelVersionClick,
  CPUMemoryTick,
  CPUModelClick,
  ProcessTableClick,
}
