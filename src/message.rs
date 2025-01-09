use iced::event::Event;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(Event),
  WeatherTick,
  WeatherClick,
  MachineInfoKernelVersionClick,
  CPUMemoryTick,
  CPUModelClick,
  ProcessTableClick,
  DiskTick,
  DiskModelClick,
  GPUTick,
  GPUModelClick,
  NetworkTick,
  NetworkWanIPTick,
  NetworkWanIPAssign(Option<IpAddr>),
  NetworkWanIPClick,
  NetworkLocalIPClick,
}
