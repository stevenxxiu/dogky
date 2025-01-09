use iced::widget::text;
use iced::{widget::Text, Color};

#[macro_export]
macro_rules! space_row {
  ($row:expr) => {
    $row.spacing($crate::styles::H_GAP)
  };
}

pub(crate) use space_row;

#[macro_export]
macro_rules! expand_right {
  ($child:expr) => {
    $child.width(Length::Fill).align_x(Horizontal::Right)
  };
}

pub(crate) use expand_right;

pub struct WithStyle {
  pub color: Color,
}

impl WithStyle {
  pub fn new(color: Color) -> Self {
    Self { color }
  }

  pub fn text<'a>(&self, s: impl text::IntoFragment<'a>) -> Text<'a> {
    text(s).color(self.color)
  }
}
