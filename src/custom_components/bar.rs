use gtk::cairo::Context;
use gtk::gdk::RGBA;
use gtk::glib::{Continue, Receiver};
use gtk::prelude::{DrawingAreaExtManual, WidgetExt};
use gtk::DrawingArea;
use std::cell::Cell;
use std::sync::Arc;

use crate::gtk_utils;

const BORDER_WIDTH: u32 = 1;

pub fn build_bar(drawing_area: DrawingArea, receiver: Receiver<f32>, border_color: &RGBA, fill_color: &RGBA) {
  let (min_x, _max_x, inner_width, min_y, _max_y, inner_height) =
    gtk_utils::get_drawing_area_inner_dims(&drawing_area, BORDER_WIDTH);
  let filled_amount = Arc::new(Cell::new(0f32));

  let [bord_r, bord_g, bord_b, bord_a] = gtk_utils::get_color_components(border_color);
  let [fill_r, fill_g, fill_b, fill_a] = gtk_utils::get_color_components(fill_color);

  let filled_amount_ = filled_amount.clone();
  drawing_area.set_draw_func(
    move |_drawing_area: &DrawingArea, context: &Context, width: i32, height: i32| {
      // Draw bar
      context.set_source_rgba(fill_r, fill_g, fill_b, fill_a);
      let x_offset = (inner_width + 1) as f64 * filled_amount_.get() as f64;
      let x_begin = (min_x - 1) as f64;
      context.rectangle(x_begin, min_y as f64, x_begin + x_offset, inner_height as f64);
      context.fill().unwrap();

      // Draw border
      context.set_source_rgba(bord_r, bord_g, bord_b, bord_a);
      gtk_utils::draw_border_rect(&context, width, height, BORDER_WIDTH);
    },
  );

  let filled_amount_ = filled_amount.clone();
  receiver.attach(None, move |cur_filled_amount| {
    filled_amount_.set(cur_filled_amount);
    drawing_area.queue_draw();
    Continue(true)
  });
}
