use iced::event::{self, Event};
use iced::widget::{self, column, Column};
use iced::window::{self, Mode};
use iced::{Point, Size, Subscription, Task};

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
