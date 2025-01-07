use iced::mouse;
use iced::widget::canvas::{self, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug, Default)]
pub struct Bar {
  pub value: f32,
  pub width: f32,
  pub height: f32,
  pub fill_color: Color,
  pub border_color: Color,
  pub cache: canvas::Cache,
}

const BORDER_WIDTH: f32 = 1.0;

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
      let inner_width = self.width - 2.0 * BORDER_WIDTH;
      let inner_height = self.height - 2.0 * BORDER_WIDTH;
      let inner_origin = Point::new(BORDER_WIDTH, BORDER_WIDTH);
      frame.fill_rectangle(
        inner_origin,
        Size::new(self.value * inner_width, inner_height),
        self.fill_color,
      );
      let stroke = Stroke::default().with_color(self.border_color);
      frame.stroke_rectangle(Point::ORIGIN, Size::new(self.width - 1.0, self.height - 1.0), stroke);
    });
    vec![geometry]
  }
}
