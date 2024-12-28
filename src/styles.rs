use iced::application::Appearance;
use iced::Color;

pub fn get_window_appearance() -> Appearance {
  Appearance {
    background_color: Color::from_rgba8(32, 32, 32, 0.27),
    text_color: Color::WHITE,
  }
}
