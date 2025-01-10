use iced::application::Appearance;
use iced::event::{self, Event};
use iced::widget::{self, column, container, horizontal_rule};
use iced::window::{self, Mode};
use iced::{Element, Font, Pixels, Point, Settings, Size, Subscription, Task};
use styles_config::{GlobalStyles, StylesConfig};
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
mod style_utils;
mod styles_config;
mod ui_utils;
mod utils;

fn set_pos_to_res(_window: Size<f32>, resolution: Size<f32>) -> Point<f32> {
  Point::new(resolution.width, resolution.height)
}

struct Dogky {
  styles: StylesConfig,
  weather: WeatherComponent,
  machine_info: MachineInfoComponent,
  cpu_memory: CpuMemoryComponent,
  disk: DiskComponent,
  gpu: GpuComponent,
  network: NetworkComponent,
}

impl Dogky {
  fn new(settings: Settings, styles: StylesConfig) -> (Self, Task<Message>) {
    let config = config::load_config().unwrap();
    let global_styles = GlobalStyles {
      container_width: styles.width as f32 - styles.padding.left - styles.padding.right,
      h_gap: styles.h_gap,
      border_width: styles.border_width,
      char_dims: style_utils::get_char_dims(&settings),
    };
    (
      Self {
        styles: styles.clone(),
        weather: WeatherComponent::new(config.weather, global_styles.clone(), styles.weather),
        machine_info: MachineInfoComponent::new(global_styles.clone(), styles.machine_info),
        cpu_memory: CpuMemoryComponent::new(config.cpu_memory, global_styles.clone(), styles.cpu_memory),
        disk: DiskComponent::new(config.disk, global_styles.clone(), styles.disk),
        gpu: GpuComponent::new(config.gpu, global_styles.clone(), styles.gpu),
        network: NetworkComponent::new(config.network, global_styles.clone(), styles.network),
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
          let size = Size::new(self.styles.width as f32, height);
          let pos = Point {
            x: (width - self.styles.width as f32),
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
    let separator = || container(horizontal_rule(1)).padding(self.styles.separator_padding.clone());
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
    .padding(self.styles.padding.clone())
    .into()
  }
}

pub fn main() {
  let styles = styles_config::load_config().unwrap();
  let settings = Settings {
    id: Some("dogky".to_string()),
    fonts: vec![],
    default_font: Font::with_name("Liberation Mono"), // Requires `&'static str`
    default_text_size: Pixels(styles.text_size),
    antialiasing: styles.antialiasing,
  };

  let appearance = Appearance {
    background_color: *styles.background_color.clone(),
    text_color: *styles.text_color.clone(),
  };
  let _ = iced::application("dogky", Dogky::update, Dogky::view)
    .subscription(Dogky::subscription)
    .settings(settings.clone())
    .window(iced::window::Settings {
      position: window::Position::SpecificWith(set_pos_to_res),
      visible: false,
      decorations: false,
      transparent: true,
      ..Default::default()
    })
    .level(window::Level::AlwaysOnBottom)
    .style(move |_state, _theme| appearance)
    .run_with(|| Dogky::new(settings, styles));
}
