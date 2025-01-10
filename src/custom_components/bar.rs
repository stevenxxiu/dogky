use iced::mouse;
use iced::widget::canvas::{self, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug, Default)]
pub struct Bar {
  pub value: f32,
  pub width: f32,
  pub height: f32,
  pub fill_color: Color,
  pub border_width: f32,
  pub border_color: Color,
  pub cache: canvas::Cache,
}

impl<Message> canvas::Program<Message> for Bar {
  type State = ();

  fn draw(
    &self,
    _state: &(),
    renderer: &Renderer,
    _theme: &Theme,
    bounds: Rectangle,
    _cursor: mouse::Cursor,
  ) -> Vec<canvas::Geometry> {
    let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
      let inner_width = self.width - 2.0 * self.border_width;
      let inner_height = self.height - 2.0 * self.border_width;
      let inner_origin = Point::new(self.border_width, self.border_width);
      frame.fill_rectangle(
        inner_origin,
        Size::new(self.value * inner_width, inner_height),
        self.fill_color,
      );
      let stroke = Stroke::default()
        .with_width(self.border_width)
        .with_color(self.border_color);
      frame.stroke_rectangle(
        Point::new(self.border_width / 2.0, self.border_width / 2.0),
        Size::new(self.width - self.border_width, self.height - self.border_width),
        stroke,
      );
    });
    vec![geometry]
  }
}
