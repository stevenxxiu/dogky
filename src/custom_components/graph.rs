use gtk::cairo::Context;
use gtk::gdk::RGBA;
use gtk::glib::{Continue, Receiver};
use gtk::prelude::{DrawingAreaExtManual, WidgetExt};
use gtk::DrawingArea;
use std::cell::Cell;
use std::sync::{Arc, Mutex};

use crate::gtk_utils;

const MAX_NUM_VALUES: usize = 500;
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

  let values = Arc::new(Mutex::new([0f32; MAX_NUM_VALUES]));
  let num_values = (max_x - min_x + 1) as usize;
  let next_index = Arc::new(Cell::new(0usize));

  let [bord_r, bord_g, bord_b, bord_a] = gtk_utils::get_color_components(border_color);
  let [graph_r, graph_g, graph_b, graph_a] = gtk_utils::get_color_components(graph_color);

  let values_ = values.clone();
  let next_index_ = next_index.clone();
  drawing_area.set_draw_func(
    move |_drawing_area: &DrawingArea, context: &Context, width: i32, height: i32| {
      let values_ = values_.lock().unwrap();
      // Draw graph
      context.set_source_rgba(graph_r, graph_g, graph_b, graph_a);
      let mut i = next_index_.get();
      let mut x = max_x;
      loop {
        if i == 0 {
          i = num_values;
        }
        i -= 1;
        let value = values_[i];
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

  let values_ = values.clone();
  let next_index_ = next_index.clone();
  receiver.attach(None, move |cur_value| {
    let mut values_ = values_.lock().unwrap();
    values_[next_index_.get()] = cur_value;

    next_index_.set(next_index_.get() + 1);
    if next_index_.get() >= num_values {
      next_index_.set(0);
    }

    drawing_area.queue_draw();
    Continue(true)
  });
}
