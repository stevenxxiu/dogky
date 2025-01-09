use iced::event::{self, Event};
use iced::widget::{self, column, container, horizontal_rule};
use iced::window::{self, Mode};
use iced::{Element, Point, Size, Subscription, Task};
use xcb::{x, Xid, XidNew};

use components::{
  CpuMemoryComponent, DiskComponent, GpuComponent, MachineInfoComponent, NetworkComponent, WeatherComponent,
};
use message::Message;

mod api;
mod components;
mod config;
mod custom_components;
mod format_size;
mod message;
mod path;
mod serde_structs;
mod styles;
mod ui_utils;
mod utils;

fn set_pos_to_res(_window: Size<f32>, resolution: Size<f32>) -> Point<f32> {
  Point::new(resolution.width, resolution.height)
}

struct Dogky {
  width: u32,
  weather: WeatherComponent,
  machine_info: MachineInfoComponent,
  cpu_memory: CpuMemoryComponent,
  disk: DiskComponent,
  gpu: GpuComponent,
  network: NetworkComponent,
}

impl Dogky {
  fn new() -> (Self, Task<Message>) {
    let config = config::load_config().unwrap();
    let padding = styles::get_padding();
    let container_width = config.width as f32 - padding.left - padding.right;
    (
      Self {
        width: config.width,
        weather: WeatherComponent::new(config.weather),
        machine_info: MachineInfoComponent::new(),
        cpu_memory: CpuMemoryComponent::new(config.cpu_memory, container_width),
        disk: DiskComponent::new(config.disk, container_width),
        gpu: GpuComponent::new(config.gpu),
        network: NetworkComponent::new(config.network, container_width),
      },
      widget::focus_next(),
    )
  }

  fn set_wm_states(window_id: u32) {
    let (conn, _screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(0).unwrap(); // Assume screen 0
    let window: x::Window = unsafe { XidNew::new(window_id) };

    let get_net_wm_atom = |name| {
      let cookie = conn.send_request(&x::InternAtom {
        only_if_exists: true,
        name,
      });
      conn.wait_for_reply(cookie).unwrap().atom()
    };

    let wm_state = get_net_wm_atom(b"_NET_WM_STATE");
    for name in [
      b"_NET_WM_STATE_STICKY".as_ref(),
      b"_NET_WM_STATE_SKIP_TASKBAR".as_ref(),
      b"_NET_WM_STATE_SKIP_PAGER".as_ref(),
      b"_NET_WM_STATE_BELOW".as_ref(),
    ] {
      let atom = get_net_wm_atom(name);
      let data = x::ClientMessageData::Data32([1, atom.resource_id(), 0, 0, 0]);
      let event = x::ClientMessageEvent::new(window, wm_state, data);
      let cookie = conn.send_request_checked(&x::SendEvent {
        propagate: false,
        destination: x::SendEventDest::Window(screen.root()),
        event_mask: x::EventMask::STRUCTURE_NOTIFY,
        event: &event,
      });
      conn.check_request(cookie).unwrap();
    }
  }

  fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::EventOccurred(event) => match event {
        Event::Window(window::Event::Opened { position, size: _ }) => {
          let position = position.unwrap();
          let (width, height) = (position.x, position.y);
          let size = Size::new(self.width as f32, height);
          let pos = Point {
            x: (width - self.width as f32),
            y: 0.,
          };
          window::get_latest().and_then(move |id| {
            Task::batch([
              window::resize(id, size),
              window::move_to(id, pos),
              window::get_raw_id::<Message>(id).then(|raw_id| {
                Self::set_wm_states(raw_id as u32);
                Task::none()
              }),
              window::change_mode(id, Mode::Windowed),
            ])
          })
        }
        _ => Task::none(),
      },
      Message::Weather(_) => self.weather.update(message),
      Message::MachineInfo(_) => self.machine_info.update(message),
      Message::CPUMemory(_) => self.cpu_memory.update(message),
      Message::Disk(_) => self.disk.update(message),
      Message::GPU(_) => self.gpu.update(message),
      Message::Network(_) => self.network.update(message),
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    Subscription::batch([
      event::listen().map(Message::EventOccurred),
      self.weather.subscription(),
      self.cpu_memory.subscription(),
      self.disk.subscription(),
      self.gpu.subscription(),
      self.network.subscription(),
    ])
  }

  fn view(&self) -> Element<Message> {
    let separator = || container(horizontal_rule(1)).padding(styles::get_separator_padding());
    column![
      self.weather.view(),
      separator(),
      self.machine_info.view(),
      separator(),
      self.cpu_memory.view(),
      separator(),
      self.disk.view(),
      separator(),
      self.gpu.view(),
      separator(),
      self.network.view()
    ]
    .padding(styles::get_padding())
    .into()
  }
}

pub fn main() {
  let _ = iced::application("dogky", Dogky::update, Dogky::view)
    .subscription(Dogky::subscription)
    .settings(styles::get_settings())
    .window(iced::window::Settings {
      position: window::Position::SpecificWith(set_pos_to_res),
      visible: false,
      decorations: false,
      transparent: true,
      ..Default::default()
    })
    .level(window::Level::AlwaysOnBottom)
    .style(|_state, _theme| styles::WINDOW_APPEARANCE)
    .run_with(Dogky::new);
}
