use std::time::Duration;

use async_io::Timer;
use circular_queue::CircularQueue;
use freya::prelude::*;
use freya::text_edit::Clipboard;
use futures_lite::stream::StreamExt;
use getifs::Ifv4Net;
use join_string::Join;
use public_ip::dns::GOOGLE_V6;
use regex::Regex;
use sysinfo::Networks;

use crate::config::NetworkConfig;
use crate::custom_components::create_graph;
use crate::format_size::{format_size, format_speed};
use crate::freya_utils::{
  border_fill_width, color_label, cursor_area, flex_cont_factory, label_with_value_factory, right_value_label,
};
use crate::styles_config::{GlobalStyles, NetworkStyles};

#[derive(Default, Clone, Debug)]
struct NetworkData {
  network_name: String,
  local_ips: Vec<Ifv4Net>,
  total_received: u64,
  total_transmitted: u64,
  upload_speed: f32,
  download_speed: f32,
}

/// Collect into a vector, so we can test if there's no IP, or to print them
fn get_local_ips(network_name: &str) -> Vec<Ifv4Net> {
  getifs::interfaces()
    .unwrap()
    .into_iter()
    .find_map(|interface| (interface.name() == network_name).then_some(interface.ipv4_addrs().unwrap().to_vec()))
    .unwrap_or_default()
}

fn get_network_data(networks: &mut Networks, interface_regex: Regex, update_interval: u64) -> Option<NetworkData> {
  let mut res = NetworkData::default();
  networks.refresh(true);

  let (network_name, network_data) = networks
    .into_iter()
    .find(|(interface_name, _data)| interface_regex.is_match(interface_name))?;

  res.local_ips = get_local_ips(network_name);
  if res.local_ips.is_empty() {
    return None;
  }
  res.network_name = network_name.to_string();

  res.total_received = network_data.total_received();
  res.total_transmitted = network_data.total_transmitted();
  res.download_speed = network_data.received() as f32 / update_interval as f32;
  res.upload_speed = network_data.transmitted() as f32 / update_interval as f32;

  Some(res)
}

fn network_graphs_component(
  download_hist: CircularQueue<f32>,
  upload_hist: CircularQueue<f32>,
  styles: NetworkStyles,
) -> Element {
  let flex_cont = flex_cont_factory(styles.graph_h_gap);
  flex_cont(vec![
    rect()
      .width(Size::flex(1.))
      .height(Size::px(styles.graph_height))
      .border(border_fill_width(
        *styles.graph_download_border_color,
        styles.graph_download_border_width,
      ))
      .child(create_graph(
        [download_hist],
        [(*styles.graph_download_fill_color).into()],
      ))
      .into(),
    rect()
      .width(Size::flex(1.))
      .height(Size::px(styles.graph_height))
      .border(border_fill_width(
        *styles.graph_upload_border_color,
        styles.graph_upload_border_width,
      ))
      .child(create_graph([upload_hist], [(*styles.graph_upload_fill_color).into()]))
      .into(),
  ])
  .into()
}

const NETWORK_DECIMAL_PLACES: usize = 2usize;

pub fn network_component() -> Element {
  let config = use_consume::<NetworkConfig>();
  let styles = use_consume::<NetworkStyles>();
  let global_styles = use_consume::<GlobalStyles>();

  let mut networks = Networks::new();

  let mut data = use_state(NetworkData::default);

  let hist_size = ((global_styles.container_width - styles.graph_h_gap) / 2.) as usize;
  let mut download_hist = use_state(|| CircularQueue::with_capacity(hist_size));
  let mut upload_hist = use_state(|| CircularQueue::with_capacity(hist_size));

  let mut public_ip_str = use_state(|| "".to_string());
  let mut local_ips_str = use_state(|| "".to_string());

  use_hook(|| {
    spawn(async move {
      loop {
        if let Some(cur_data) =
          get_network_data(&mut networks, (*config.interface_regex).clone(), config.update_interval)
        {
          let graph_config = &config.graphs;
          let download_ratio = cur_data.download_speed / graph_config.download.maximum_bytes_per_sec as f32;
          download_hist.write().push(download_ratio);
          let upload_ratio = cur_data.upload_speed / graph_config.upload.maximum_bytes_per_sec as f32;
          upload_hist.write().push(upload_ratio);

          // Only include IPv4, as IPv6 addresses are too long
          local_ips_str.set(
            cur_data
              .local_ips
              .iter()
              .map(|ip| ip.addr().to_string())
              .join(" ")
              .to_string(),
          );

          data.set(cur_data);
        }
        Timer::interval(Duration::from_secs(config.update_interval))
          .next()
          .await;
      }
    });
    if let Some(interval) = config.public_ip_retry_timeout {
      spawn(async move {
        loop {
          if let Some(ip) = public_ip::addr_with(GOOGLE_V6, public_ip::Version::V6).await {
            public_ip_str.set(ip.to_string());
          }
          Timer::interval(Duration::from_secs(interval)).next().await;
        }
      });
    }
  });

  let value_color: Color = (*styles.value_color).into();
  let flex_cont = flex_cont_factory(global_styles.h_gap);
  let label_with_value = label_with_value_factory((*styles.name_color).into(), value_color);

  if data.read().local_ips.is_empty() {
    color_label(*styles.name_color, "Disconnected").into()
  } else {
    let data_ = data.read();
    let [down_speed, up_speed] = [data_.download_speed, data_.upload_speed];
    let [down_total, up_total] = [data_.total_received, data_.total_transmitted];
    rect()
      .children([
        flex_cont(vec![
          color_label(*styles.name_color, "WAN IP").into(),
          cursor_area(CursorIcon::Copy)
            .child(
              right_value_label(value_color, public_ip_str.read().clone())
                .on_mouse_down(move |_| Clipboard::set(public_ip_str.read().clone()).unwrap()),
            )
            .into(),
        ])
        .into(),
        flex_cont(vec![
          color_label(*styles.name_color, format!("{} IP", data.read().network_name)).into(),
          cursor_area(CursorIcon::Copy)
            .child(
              right_value_label(value_color, local_ips_str.read().clone()).on_mouse_down(move |_| {
                let local_ips = &data.read().local_ips;
                Clipboard::set(local_ips.iter().map(|ip| ip.to_string()).join("\n").to_string()).unwrap();
              }),
            )
            .into(),
        ])
        .into(),
        flex_cont(vec![
          label_with_value("Net Down", format_speed(down_speed, NETWORK_DECIMAL_PLACES)).into(),
          label_with_value("Net Up", format_speed(up_speed, NETWORK_DECIMAL_PLACES)).into(),
        ])
        .into(),
        flex_cont(vec![
          label_with_value("Total Down", format_size(down_total, NETWORK_DECIMAL_PLACES)).into(),
          label_with_value("Total Up", format_size(up_total, NETWORK_DECIMAL_PLACES)).into(),
        ])
        .into(),
        network_graphs_component((*download_hist.read()).clone(), (*upload_hist.read()).clone(), styles),
      ])
      .into()
  }
}
