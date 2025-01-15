use freya::prelude::*;
use skia_safe::{Color, Paint};

#[allow(non_snake_case)]
#[component]
pub fn Separator() -> Element {
  let platform = use_platform();
  let (_reference, size) = use_node_signal();
  let separator = use_canvas(move || {
    platform.invalidate_drawing_area(size.peek().area);
    Box::new(move |ctx| {
      let mut paint = Paint::default();
      paint.set_style(skia_safe::paint::Style::Stroke);
      paint.set_stroke_width(1.0);
      paint.set_color(Color::from_rgb(203, 203, 203));

      ctx.canvas.draw_line(
        (ctx.area.min_x(), ctx.area.min_y()),
        (ctx.area.max_x(), ctx.area.min_y()),
        &paint,
      );
    })
  });

  rsx!(
    rect {
      width: "100%",
      height: "18",
      main_align: "center",
      rect {
        width: "100%",
        height: "1",
        canvas_reference: separator.attribute(),
      }
    },
  )
}
