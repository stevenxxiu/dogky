use circular_queue::CircularQueue;
use iced::mouse;
use iced::widget::canvas::path::Builder;
use iced::widget::canvas::{self, Stroke};
use iced::{Color, Point, Rectangle, Renderer, Size, Theme};

#[derive(Debug)]
pub struct Graph {
  pub datasets: Vec<CircularQueue<f32>>,
  pub width: f32,
  pub height: f32,
  pub border_width: f32,
  pub border_color: Color,
  pub graph_colors: Vec<Color>,
  pub cache: canvas::Cache,
}

impl<Message> canvas::Program<Message> for Graph {
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
      let inner_height = self.height - 2.0 * self.border_width;
      let min_x = self.border_width;
      let max_y = self.height - self.border_width;

      // Draw graph
      for (i, (dataset, color)) in self.datasets.iter().zip(self.graph_colors.iter()).enumerate() {
        let mut graph_path = Builder::new();
        let mut x = self.width - self.border_width;
        if i != 0 {
          if let Some(value) = dataset.iter().next() {
            let height = inner_height * value;
            graph_path.move_to(Point::new(x, max_y - height));
          }
        }
        for value in dataset.iter() {
          let height = inner_height * value;
          if i == 0 {
            graph_path.rectangle(
              Point::new(x - self.border_width, max_y - height),
              Size::new(self.border_width, height),
            );
          } else {
            graph_path.line_to(Point::new(x - self.border_width, max_y - height));
          }
          x -= 1.0;
          if x < min_x {
            break;
          }
        }
        if i == 0 {
          frame.fill(&graph_path.build(), *color);
        } else {
          let stroke = Stroke::default().with_color(*color);
          frame.stroke(&graph_path.build(), stroke);
        }
      }

      // Draw border
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
