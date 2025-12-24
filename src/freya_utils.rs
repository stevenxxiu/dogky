use std::borrow::Cow;

use freya::prelude::*;

pub fn cursor_area(icon: CursorIcon) -> CursorArea {
  CursorArea::new().icon(icon)
}

fn horizontal_cont(h_gap: f32) -> Rect {
  rect()
    .width(Size::percent(100.))
    .direction(Direction::Horizontal)
    .spacing(h_gap)
}

pub fn horizontal_cont_factory<V>(h_gap: f32) -> impl Fn(V) -> Rect
where
  V: Into<Vec<Element>>,
{
  move |children: V| horizontal_cont(h_gap).children(children)
}

pub fn center_cont_factory<V>(h_gap: f32) -> impl Fn(V) -> Rect
where
  V: Into<Vec<Element>>,
{
  move |children: V| horizontal_cont(h_gap).main_align(Alignment::Center).children(children)
}

pub fn flex_cont_factory<V>(h_gap: f32) -> impl Fn(V) -> Rect
where
  V: Into<Vec<Element>>,
{
  move |children: V| horizontal_cont(h_gap).content(Content::Flex).children(children)
}

pub fn value_label_factory<S>(color: Color) -> impl Fn(S) -> Label
where
  S: Into<Cow<'static, str>>,
{
  move |text: S| label().color(color).text(text)
}

pub fn right_value_label(color: Color, text: impl Into<Cow<'static, str>>) -> Label {
  label()
    .width(Size::flex(1.))
    .text_align(TextAlign::Right)
    .color(color)
    .text(text)
}

pub fn label_with_value_factory<S>(label_color: Color, value_color: Color) -> impl Fn(S, String) -> Rect
where
  S: Into<Cow<'static, str>>,
{
  move |label_text: S, value: String| {
    rect().width(Size::flex(1.)).direction(Direction::Horizontal).children([
      label().color(label_color).text(label_text).into(),
      right_value_label(value_color, value).into(),
    ])
  }
}

pub fn color_label(color: impl Into<Color>, text: impl Into<Cow<'static, str>>) -> Label {
  label().color(color).text(text)
}

pub fn emoji_label(text: impl Into<Cow<'static, str>>) -> Label {
  label().font_family("Noto Color Emoji").text(text)
}

pub fn border_fill_width(color: impl Into<Color>, width: f32) -> Border {
  Border::new().fill(color).width(width)
}
