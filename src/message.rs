use iced::event::Event;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub enum Message {
  EventOccurred(Event),
  Weather(WeatherMessage),
  MachineInfo(MachineInfoMessage),
  CPUMemory(CPUMemoryMessage),
  Disk(DiskMessage),
  GPU(GPUMessage),
  Network(NetworkMessage),
}

#[derive(Debug, Clone)]
pub enum WeatherMessage {
  Tick,
  Click,
}

#[derive(Debug, Clone)]
pub enum MachineInfoMessage {
  KernelVersionClick,
}

#[derive(Debug, Clone)]
pub enum CPUMemoryMessage {
  Tick,
  CPUModelClick,
  ProcessTableClick,
}

#[derive(Debug, Clone)]
pub enum DiskMessage {
  Tick,
  ModelClick,
}

#[derive(Debug, Clone)]
pub enum GPUMessage {
  Tick,
  ModelClick,
}

#[derive(Debug, Clone)]
pub enum NetworkMessage {
  Tick,
  WanIPTick,
  WanIPAssign(Option<IpAddr>),
  WanIPClick,
  LocalIPClick,
}
