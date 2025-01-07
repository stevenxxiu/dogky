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
  pub border_color: Color,
  pub graph_colors: Vec<Color>,
  pub cache: canvas::Cache,
}

const BORDER_WIDTH: f32 = 1.0;

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
      let inner_height = self.height - 2.0 * BORDER_WIDTH;
      let min_x = BORDER_WIDTH;
      let max_y = self.height - 2.0 * BORDER_WIDTH;

      // Draw graph
      for (i, (dataset, color)) in self.datasets.iter().zip(self.graph_colors.iter()).enumerate() {
        let mut graph_path = Builder::new();
        let mut x = self.width - BORDER_WIDTH;
        if i != 0 {
          if let Some(value) = dataset.iter().next() {
            let height = inner_height * value;
            graph_path.move_to(Point::new(x, max_y - height));
          }
        }
        for value in dataset.iter() {
          let height = inner_height * value;
          if i == 0 {
            graph_path.rectangle(Point::new(x - 1.0, max_y - height), Size::new(1.0, height));
          } else {
            graph_path.line_to(Point::new(x - 1.0, max_y - height));
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
      let stroke = Stroke::default().with_color(self.border_color);
      frame.stroke_rectangle(Point::ORIGIN, Size::new(self.width - 1.0, self.height - 1.0), stroke);
    });
    vec![geometry]
  }
}
