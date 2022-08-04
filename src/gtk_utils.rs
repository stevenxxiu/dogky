use gtk::cairo::Context;
use gtk::prelude::DrawingAreaExt;
use gtk::DrawingArea;

pub fn get_drawing_area_inner_dims(drawing_area: &DrawingArea, border_width: u32) -> (i32, i32, i32, i32, i32, i32) {
  let min_x = border_width as i32;
  let max_x = drawing_area.content_width() - border_width as i32;
  let inner_width = max_x - min_x;

  let min_y = border_width as i32;
  let max_y = drawing_area.content_height() - border_width as i32;
  let inner_height = max_y - min_y;

  (min_x, max_x, inner_width, min_y, max_y, inner_height)
}

pub fn draw_border_rect(context: &Context, width: i32, height: i32, border_width: u32) {
  let border_width = border_width as f64;
  context.set_line_width(border_width);
  context.rectangle(
    border_width / 2.0,
    border_width / 2.0,
    width as f64 - border_width,
    height as f64 - border_width,
  );
  context.stroke().unwrap();
}