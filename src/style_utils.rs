use iced::{Settings, Size};

pub fn get_char_dims(app_settings: &Settings) -> Size {
  use iced::advanced::text::Paragraph as _;
  use iced::advanced::text::{LineHeight, Shaping, Wrapping};
  use iced::advanced::Text;
  use iced::alignment::{Horizontal, Vertical};
  use iced_graphics::text::paragraph::Paragraph;

  let cur_text = Text {
    content: " ",
    bounds: iced::Size::INFINITY,
    font: app_settings.default_font,
    size: app_settings.default_text_size,
    horizontal_alignment: Horizontal::Left,
    vertical_alignment: Vertical::Top,
    line_height: LineHeight::Relative(1.),
    shaping: Shaping::Basic,
    wrapping: Wrapping::None,
  };
  let paragraph = Paragraph::with_text(cur_text);
  paragraph.min_bounds()
}
