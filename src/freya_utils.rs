use std::borrow::Cow;

use freya::prelude::*;

pub fn cursor_area(icon: CursorIcon) -> CursorArea {
  CursorArea::new().icon(icon)
}

pub struct HorizontalCont {
  h_gap: f32,
}

pub fn horizontal_cont(h_gap: f32) -> HorizontalCont {
  HorizontalCont { h_gap }
}

impl HorizontalCont {
  pub fn children(&self, children: impl IntoIterator<Item = impl IntoElement>) -> Rect {
    rect()
      .width(Size::percent(100.))
      .direction(Direction::Horizontal)
      .spacing(self.h_gap)
      .children(children)
  }
}

pub struct CenterCont {
  h_gap: f32,
}

pub fn center_cont(h_gap: f32) -> CenterCont {
  CenterCont { h_gap }
}

impl CenterCont {
  pub fn children(&self, children: impl IntoIterator<Item = impl IntoElement>) -> Rect {
    horizontal_cont(self.h_gap)
      .children(children)
      .main_align(Alignment::Center)
  }
}

pub struct FlexCont {
  h_gap: f32,
}

pub fn flex_cont(h_gap: f32) -> FlexCont {
  FlexCont { h_gap }
}

impl FlexCont {
  pub fn children(&self, children: impl IntoIterator<Item = impl IntoElement>) -> Rect {
    horizontal_cont(self.h_gap).children(children).content(Content::Flex)
  }
}

pub fn value_label_factory<C: Into<Color>, S>(color: C) -> impl Fn(S) -> Label
where
  S: Into<Cow<'static, str>>,
{
  let color = color.into();
  move |text: S| label().color(color).text(text)
}

pub fn right_value_label<C: Into<Fill>>(color: C, text: impl Into<Cow<'static, str>>) -> Label {
  label()
    .width(Size::flex(1.))
    .text_align(TextAlign::Right)
    .color(color)
    .text(text)
}

pub fn label_with_value_factory<C1, C2, S>(label_color: Option<C1>, value_color: C2) -> impl Fn(S, String) -> Rect
where
  C1: Into<Color>,
  C1: Copy,
  C2: Into<Color>,
  S: Into<Cow<'static, str>>,
{
  let value_color = value_color.into();
  move |label_text: S, value: String| {
    let mut left_label = label().text(label_text);
    if let Some(label_color) = &label_color {
      left_label = left_label.color((*label_color).into());
    }
    rect()
      .width(Size::flex(1.))
      .direction(Direction::Horizontal)
      .children([left_label, right_value_label(value_color, value)])
  }
}

pub fn color_label(color: impl Into<Fill>, text: impl Into<Cow<'static, str>>) -> Label {
  label().color(color).text(text)
}

pub fn emoji_label(text: impl Into<Cow<'static, str>>) -> Label {
  label().font_family("Noto Color Emoji").text(text)
}

pub fn border_fill_width(color: impl Into<Color>, width: f32) -> Border {
  Border::new().fill(color).width(width)
}
