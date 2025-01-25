use circular_queue::CircularQueue;
use freya::prelude::*;
use skia_safe::{self as sk};

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

      let (min_x, max_x, min_y, max_y) = (
        ctx.area.min_x() + BORDER_WIDTH,
        ctx.area.max_x() - BORDER_WIDTH,
        ctx.area.min_y() + BORDER_WIDTH,
        ctx.area.max_y() - BORDER_WIDTH,
      );
      let height = max_y - min_y;

      for (i, (dataset, &color)) in datasets.iter().zip(graph_colors.iter()).enumerate() {
        let mut x = max_x;
        if i == 0 {
          paint.set_style(skia_safe::paint::Style::Fill);
          paint.set_color(color);
        } else if i == 1 {
          paint.set_style(skia_safe::paint::Style::Stroke);
          paint.set_color(color);
        }
        let mut data_iter = dataset.iter();
        let mut prev_point: sk::Point = sk::Point::new(0., 0.);
        if i >= 1 {
          if let Some(value) = data_iter.next() {
            prev_point = sk::Point::new(x, max_y - value * height);
            x -= 1.;
          }
        }
        for value in data_iter {
          if i == 0 {
            ctx
              .canvas
              .draw_rect(sk::Rect::new(x - 1., max_y - height * value, x, max_y), &paint);
          } else {
            let point = sk::Point::new(x - 1., max_y - height * value);
            ctx.canvas.draw_line(point, prev_point, &paint);
            prev_point = point;
          }
          x -= 1.;
          if x < min_x {
            break;
          }
        }
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
