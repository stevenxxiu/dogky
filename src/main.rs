use iced::event::{self, Event};
use iced::widget::{self, column, Column};
use iced::window::{self, Mode};
use iced::{Point, Size, Subscription, Task};
use xcb::{x, Xid, XidNew};

mod config;
mod path;
mod serde_structs;
mod styles;

fn set_pos_to_res(_window: Size<f32>, resolution: Size<f32>) -> Point<f32> {
  Point::new(resolution.width, resolution.height)
}

#[derive(Debug, Clone)]
enum Message {
  EventOccurred(Event),
}

#[derive(Default)]
struct Dogky {
  config_props: Option<config::ConfigProps>,
}

impl Dogky {
  fn new() -> (Self, Task<Message>) {
    (
      Self {
        config_props: Some(config::load_config().unwrap()),
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
    let config_props = self.config_props.as_ref().unwrap();
    match message {
      Message::EventOccurred(event) => match event {
        Event::Window(window::Event::Opened { position, size: _ }) => {
          let position = position.unwrap();
          let (width, height) = (position.x, position.y);
          let size = Size::new(config_props.width as f32, height);
          let pos = Point {
            x: (width - config_props.width as f32),
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
    }
  }

  fn subscription(&self) -> Subscription<Message> {
    event::listen().map(Message::EventOccurred)
  }

  fn view(&self) -> Column<Message> {
    column![]
  }
}

pub fn main() {
  let _ = iced::application("dogky", Dogky::update, Dogky::view)
    .subscription(Dogky::subscription)
    .antialiasing(true)
    .window(iced::window::Settings {
      position: window::Position::SpecificWith(set_pos_to_res),
      visible: false,
      decorations: false,
      transparent: true,
      ..Default::default()
    })
    .level(window::Level::AlwaysOnBottom)
    .style(|_state, _theme| styles::get_window_appearance())
    .run_with(Dogky::new);
}
