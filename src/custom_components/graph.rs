use circular_queue::CircularQueue;
use gtk::cairo::Context;
use gtk::gdk::RGBA;
use gtk::glib::{Continue, Receiver};
use gtk::prelude::{DrawingAreaExtManual, WidgetExt};
use gtk::DrawingArea;
use std::sync::{Arc, Mutex};

use crate::gtk_utils;

const BORDER_WIDTH: u32 = 1;

pub fn build_graph(
  drawing_area: DrawingArea,
  receiver: Receiver<f32>,
  max_value: f32,
  border_color: &RGBA,
  graph_color: &RGBA,
) {
  let (min_x, max_x, _inner_width, _min_y, max_y, inner_height) =
    gtk_utils::get_drawing_area_inner_dims(&drawing_area, BORDER_WIDTH);

  let num_values = (max_x - min_x + 1) as usize;
  let data = Arc::new(Mutex::new(CircularQueue::with_capacity(num_values)));

  let [bord_r, bord_g, bord_b, bord_a] = gtk_utils::get_color_components(border_color);
  let [graph_r, graph_g, graph_b, graph_a] = gtk_utils::get_color_components(graph_color);

  let data_ = data.clone();
  drawing_area.set_draw_func(
    move |_drawing_area: &DrawingArea, context: &Context, width: i32, height: i32| {
      // Draw graph
      context.set_source_rgba(graph_r, graph_g, graph_b, graph_a);
      let mut x = max_x;
      for value in data_.lock().unwrap().iter() {
        let y_offset = inner_height as f64 * (value / max_value) as f64;
        context.move_to(x as f64, max_y as f64);
        context.line_to(x as f64, max_y as f64 - y_offset);
        x -= 1;
        if x < min_x {
          break;
        }
      }
      context.stroke().unwrap();

      // Draw border
      context.set_source_rgba(bord_r, bord_g, bord_b, bord_a);
      gtk_utils::draw_border_rect(&context, width, height, BORDER_WIDTH);
    },
  );

  let data_ = data.clone();
  receiver.attach(None, move |cur_value| {
    data_.lock().unwrap().push(cur_value);
    drawing_area.queue_draw();
    Continue(true)
  });
}
