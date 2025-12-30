use std::rc::Rc;
use std::{any::Any, borrow::Cow};

use circular_queue::CircularQueue;
use freya::engine::prelude::{Paint, PaintStyle, Point};
use freya::prelude::*;
use freya_core::{element::ElementExt, tree::DiffModifies};
use freya_engine::prelude::PathBuilder;

static BORDER_WIDTH: f32 = 1.;

#[derive(PartialEq)]
pub struct Graph<const N: usize> {
  layout_data: LayoutData,
  datasets: [CircularQueue<f32>; N],
  colors: [Color; N],
}

impl<const N: usize> ElementExt for Graph<N> {
  fn diff(&self, other: &std::rc::Rc<dyn ElementExt>) -> DiffModifies {
    let Some(element) = (other.as_ref() as &dyn Any).downcast_ref::<Self>() else {
      return DiffModifies::all();
    };
    let mut diff = DiffModifies::empty();
    if self.colors != element.colors || self.datasets != element.datasets {
      diff.insert(DiffModifies::STYLE);
    }
    if self.layout_data != element.layout_data {
      diff.insert(DiffModifies::LAYOUT);
    }
    diff
  }

  fn layout(&'_ self) -> std::borrow::Cow<'_, LayoutData> {
    Cow::Borrowed(&self.layout_data)
  }

  fn render(&self, context: RenderContext) {
    let area = context.layout_node.visible_area();
    let mut paint = Paint::default();
    paint.set_anti_alias(true);

    let (min_x, max_x, min_y, max_y) = (
      area.min_x() + BORDER_WIDTH,
      area.max_x() - BORDER_WIDTH,
      area.min_y() + BORDER_WIDTH,
      area.max_y() - BORDER_WIDTH,
    );
    let height = max_y - min_y;

    for (i, (dataset, &color)) in self.datasets.iter().zip(self.colors.iter()).enumerate() {
      if i == 0 {
        paint.set_style(PaintStyle::Fill);
        paint.set_color(color);
      } else if i == 1 {
        paint.set_style(PaintStyle::Stroke);
        paint.set_color(color);
      }
      let mut path = PathBuilder::new();
      let mut x = 0f32;
      if i == 0 {
        x = max_x;
        path.move_to(Point::new(x, max_y));
      }
      for (j, value) in dataset.iter().enumerate() {
        x = max_x - j as f32;
        if x < min_x {
          break;
        }
        if i == 1 && j == 0 {
          path.move_to(Point::new(x, max_y - height * value));
        } else {
          path.line_to(Point::new(x, max_y - height * value));
        }
      }
      if i == 0 {
        path.line_to(Point::new(x, max_y));
      }
      let path = path.detach();
      context.canvas.draw_path(&path, &paint);
    }
  }
}

impl<const N: usize> LayoutExt for Graph<N> {
  fn get_layout(&mut self) -> &mut LayoutData {
    &mut self.layout_data
  }
}

impl<const N: usize> ContainerExt for Graph<N> {}

impl<const N: usize> From<Graph<N>> for Element {
  fn from(value: Graph<N>) -> Self {
    Element::Element {
      key: DiffKey::None,
      element: Rc::new(value),
      elements: Vec::new(),
    }
  }
}

pub fn create_graph<const N: usize>(datasets: [CircularQueue<f32>; N], colors: [Color; N]) -> Graph<N> {
  Graph {
    layout_data: LayoutData::default(),
    datasets,
    colors,
  }
  .width(Size::flex(1.))
  .height(Size::flex(1.))
}
