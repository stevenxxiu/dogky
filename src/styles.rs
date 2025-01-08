use iced::application::Appearance;
use iced::padding::Padding;
use iced::{color, Color, Font, Pixels, Settings, Size};

pub fn get_settings() -> Settings {
  Settings {
    id: Some("dogky".to_string()),
    fonts: vec![],
    default_font: Font::with_name("Liberation Mono"),
    default_text_size: Pixels(16.),
    antialiasing: true,
  }
}

pub fn get_char_dims() -> Size {
  use iced::advanced::text::Paragraph as _;
  use iced::advanced::text::{LineHeight, Shaping, Wrapping};
  use iced::advanced::Text;
  use iced::alignment::{Horizontal, Vertical};
  use iced_graphics::text::paragraph::Paragraph;

  let app_settings = crate::styles::get_settings();
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

pub static WINDOW_APPEARANCE: Appearance = Appearance {
  background_color: color!(32, 32, 32, 0.27),
  text_color: Color::WHITE,
};

pub fn get_separator_padding() -> Padding {
  Padding::from([8, 0])
}

pub fn get_padding() -> Padding {
  Padding::from([0, 10])
}

const MARGIN_SIDE: f32 = 4.;
pub const H_GAP: f32 = 8.;

pub mod weather {
  use super::*;

  pub const CONTAINER_PADDING: Padding = Padding {
    top: 10.,
    right: MARGIN_SIDE,
    bottom: 0.,
    left: MARGIN_SIDE,
  };

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

pub mod machine_info {
  use super::*;

  pub const USER_COLOR: Color = color!(0x9cff9f);
  pub const AT_COLOR: Color = color!(0x4251f5);
  pub const HOST_COLOR: Color = color!(0x42d1f5);
  pub const DISTRO_COLOR: Color = color!(0xdddddd);
  pub const ARCHITECTURE_COLOR: Color = color!(0xdddddd);
  pub const KERNEL_VERSION_COLOR: Color = color!(0xd9d900);
}

pub mod cpu_memory {
  use super::*;

  pub const VALUE_COLOR: Color = color!(0xdddddd);

  pub const BARS_V_GAP: f32 = 4.;
  pub const BAR_HEIGHT: f32 = 13.;
  pub const BAR_BORDER_COLOR: Color = color!(0xdddddd);
  pub const BAR_FILL_COLOR: Color = color!(0xffffff);
  pub const BAR_H_GAP: f32 = 4.;

  pub const GRAPH_H_GAP: f32 = 4.;
  pub const GRAPH_HEIGHT: f32 = 60.;
  pub const GRAPH_CPU_BORDER_COLOR: Color = color!(0x42e694);
  pub const GRAPH_CPU_FILL_COLOR: Color = color!(0x4bffa5);
  pub const GRAPH_MEMORY_BORDER_COLOR: Color = color!(0x3de9e9);
  pub const GRAPH_MEMORY_FILL_COLOR: Color = color!(0x44ffff);
  pub const GRAPH_SWAP_FILL_COLOR: Color = color!(0x994cff);

  pub const PS_HEADER_COLOR: Color = color!(0xbbbbbb);
  pub const PS_SORT_FONT: Font = Font::with_name("Noto Sans Symbols 2");
  pub const PS_SORT_CPU_COLOR: Color = color!(0x777777);
  pub const PS_SORT_MEMORY_COLOR: Color = color!(0xeeeeee);
  pub const PS_CPU_COLOR: Color = color!(0x555555);
  pub const PS_MEMORY_COLOR: Color = color!(0xeeeeee);
}

pub mod disk {
  use super::*;

  pub const NAME_COLOR: Color = color!(0xcccccc);
  pub const VALUE_COLOR: Color = color!(0xffffff);
  pub const BAR_HEIGHT: f32 = 5.;
  pub const BAR_BORDER_COLOR: Color = color!(0xdddddd);
  pub const BAR_FILL_COLOR: Color = color!(0xffffff);
}

pub mod gpu {
  use super::*;

  pub const NAME_COLOR: Color = color!(0xcccccc);
  pub const USAGE_NAME_COLOR: Color = color!(0xdddddd);
  pub const VALUE_COLOR: Color = color!(0xffffff);
}
