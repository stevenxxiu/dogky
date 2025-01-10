use iced::widget::{text, Row};
use iced::{widget::Text, Color};

#[macro_export]
macro_rules! expand_right {
  ($child:expr) => {
    $child.width(Length::Fill).align_x(Horizontal::Right)
  };
}

pub(crate) use expand_right;

pub struct WithSpacing {
  pub spacing: f32,
}

impl WithSpacing {
  pub fn new(spacing: f32) -> Self {
    Self { spacing }
  }

  pub fn row<'a, T>(&self, row: Row<'a, T>) -> Row<'a, T> {
    row.spacing(self.spacing)
  }
}

pub struct WithColor {
  pub color: Color,
}

impl WithColor {
  pub fn new(color: Color) -> Self {
    Self { color }
  }

  pub fn text<'a>(&self, s: impl text::IntoFragment<'a>) -> Text<'a> {
    text(s).color(self.color)
  }
}
