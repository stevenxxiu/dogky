use freya::prelude::*;

#[derive(Clone, PartialEq, Default)]
pub struct Separator {
  height: f32,
}

pub fn create_separator(height: f32) -> Separator {
  Separator { height }
}

impl Component for Separator {
  fn render(&self) -> impl IntoElement {
    rect()
      .width(Size::percent(100.))
      .height(Size::px(self.height))
      .main_align(Alignment::Center)
      .child(
        rect()
          .width(Size::percent(100.))
          .height(Size::px(1.))
          .background(Color::from_rgb(203, 203, 203)),
      )
  }
}
