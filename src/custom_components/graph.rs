use circular_queue::CircularQueue;
use freya::prelude::*;
use skia_safe::{self as sk, Path, PathFillType};

static BORDER_WIDTH: f32 = 1.;

#[allow(non_snake_case)]
#[component]
pub fn Graph<const N: usize>(datasets: [CircularQueue<f32>; N], graph_colors: [sk::Color; N]) -> Element {
  let platform = use_platform();
  let (reference, size) = use_node_signal();

  let graph = use_canvas_with_deps(&datasets, move |datasets| {
    platform.invalidate_drawing_area(size.peek().area);
    platform.request_animation_frame();

    Box::new(move |ctx| {
      let mut paint = sk::Paint::default();
      paint.set_anti_alias(true);

      let (min_x, max_x, min_y, max_y) = (
        ctx.area.min_x() + BORDER_WIDTH,
        ctx.area.max_x() - BORDER_WIDTH,
        ctx.area.min_y() + BORDER_WIDTH,
        ctx.area.max_y() - BORDER_WIDTH,
      );
      let height = max_y - min_y;

      for (i, (dataset, &color)) in datasets.iter().zip(graph_colors.iter()).enumerate() {
        if i == 0 {
          paint.set_style(sk::paint::Style::Fill);
          paint.set_color(color);
        } else if i == 1 {
          paint.set_style(sk::paint::Style::Stroke);
          paint.set_color(color);
        }
        let mut points: Vec<sk::Point> = vec![];
        if i == 0 {
          points.push(sk::Point::new(max_x, max_y));
        }
        for (j, value) in dataset.iter().enumerate() {
          let x = max_x - j as f32;
          if x < min_x {
            break;
          }
          points.push(sk::Point::new(x, max_y - height * value));
        }
        if i == 0 {
          points.push(sk::Point::new(points.last().unwrap().x, max_y));
        }
        let path = Path::polygon(&points, false, PathFillType::Winding, false);
        ctx.canvas.draw_path(&path, &paint);
      }
    })
  });

  rsx!(rect {
    width: "100%",
    height: "100%",
    canvas_reference: graph.attribute(),
    reference,
  })
}
