use gtk::cairo::Context;
use gtk::gdk::RGBA;
use gtk::glib::{Continue, Receiver};
use gtk::prelude::{DrawingAreaExtManual, WidgetExt};
use gtk::DrawingArea;
use std::sync::{Arc, Mutex};

use crate::gtk_utils;

const MAX_NUM_VALUES: usize = 500;
const BORDER_WIDTH: u32 = 1;

struct GraphData {
  values: [f32; MAX_NUM_VALUES],
  next_index: usize,
}

pub fn build_graph(
  drawing_area: DrawingArea,
  receiver: Receiver<f32>,
  max_value: f32,
  border_color: &RGBA,
  graph_color: &RGBA,
) {
  let (min_x, max_x, _inner_width, _min_y, max_y, inner_height) =
    gtk_utils::get_drawing_area_inner_dims(&drawing_area, BORDER_WIDTH);

  let data = Arc::new(Mutex::new(GraphData {
    values: [0f32; MAX_NUM_VALUES],
    next_index: 0,
  }));
  let num_values = (max_x - min_x + 1) as usize;

  let [bord_r, bord_g, bord_b, bord_a] = gtk_utils::get_color_components(border_color);
  let [graph_r, graph_g, graph_b, graph_a] = gtk_utils::get_color_components(graph_color);

  let data_ = data.clone();
  drawing_area.set_draw_func(
    move |_drawing_area: &DrawingArea, context: &Context, width: i32, height: i32| {
      let data = data_.lock().unwrap();
      // Draw graph
      context.set_source_rgba(graph_r, graph_g, graph_b, graph_a);
      let mut i = data.next_index;
      let mut x = max_x;
      loop {
        if i == 0 {
          i = num_values;
        }
        i -= 1;
        let value = data.values[i];
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
    let mut data = data_.lock().unwrap();
    let next_index = data.next_index;
    data.values[next_index] = cur_value;

    data.next_index += 1;
    if data.next_index >= num_values {
      data.next_index = 0;
    }

    drawing_area.queue_draw();
    Continue(true)
  });
}
