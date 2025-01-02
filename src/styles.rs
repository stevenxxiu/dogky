use iced::application::Appearance;
use iced::padding::Padding;
use iced::{color, Color, Font, Pixels, Settings};
use std::borrow::Cow;

pub fn get_settings() -> Settings {
  Settings {
    id: Some("dogky".to_string()),
    fonts: vec![Cow::Borrowed(b"Liberation Mono")],
    default_font: Font::with_name("Liberation Mono"),
    default_text_size: Pixels(16.),
    antialiasing: true,
  }
}

pub static WINDOW_APPEARANCE: Appearance = Appearance {
  background_color: color!(32, 32, 32, 0.27),
  text_color: Color::WHITE,
};

pub fn get_padding() -> Padding {
  Padding::from([0, 10])
}

const MARGIN_SIDE: f32 = 4.;

pub mod weather {
  use super::*;

  pub const CONTAINER_PADDING: Padding = Padding {
    top: 10.,
    right: MARGIN_SIDE,
    bottom: 0.,
    left: MARGIN_SIDE,
  };

  pub const SPACING: f32 = 8.;

  pub const ICON_FONT: Font = Font::with_name("Noto Color Emoji");

  pub const COND_ICON_SIZE: f32 = 33.;
  pub const COND_ICON_PADDING: Padding = iced::Padding {
    top: 0.,
    right: -40.,
    bottom: 0.,
    left: 0.,
  };

  pub const VALUE_COLOR: Color = color!(0xdddddd);

  pub const WIND_ARROW_OFFSET: f32 = 10.;
  pub const WIND_ARROW_SIZE: f32 = 20.;
  pub const WIND_ARROW_CANVAS_SIZE: f32 = 20.;

  pub const SUNRISE_ICON_PADDING: Padding = iced::Padding {
    top: 0.,
    right: -19.,
    bottom: 0.,
    left: 0.,
  };
  pub const SUNSET_ICON_PADDING: Padding = iced::Padding {
    top: 0.,
    right: 0.,
    bottom: 0.,
    left: 0.,
  };
}
