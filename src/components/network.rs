use std::net::IpAddr;

use arboard::Clipboard;
use circular_queue::CircularQueue;
use freya::prelude::*;
use join_string::Join;
use public_ip::dns::GOOGLE_V6;
use regex::Regex;
use sysinfo::Networks;

use crate::config::NetworkConfig;
use crate::custom_components::{Graph, LabelRight};
use crate::format_size::{format_size, format_speed};
use crate::styles_config::{GlobalStyles, NetworkStyles};

#[derive(Default, Clone, Debug)]
struct NetworkData {
  network_name: String,
  local_ips: Vec<IpAddr>,
  total_received: u64,
  total_transmitted: u64,
  upload_speed: f32,
  download_speed: f32,
}

/// Collect into a vector, so we can test if there's no IP, or to print them
fn get_local_ips(network_name: &str) -> Vec<IpAddr> {
  local_ip_address::list_afinet_netifas()
    .unwrap()
    .into_iter()
    .filter_map(|(name, ip)| (name == network_name).then_some(ip))
    .collect()
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

#[allow(non_snake_case)]
#[component]
fn NetworkGraphsComponent(
  download_hist: ReadOnlySignal<CircularQueue<f32>>,
  upload_hist: ReadOnlySignal<CircularQueue<f32>>,
) -> Element {
  let styles = use_context::<NetworkStyles>();
  rsx!(
    rect {
      width: "100%",
      direction: "horizontal",
      content: "flex",
      spacing: styles.graph_h_gap.to_string(),
      rect {
        width: "flex(1)",
        height: styles.graph_height.to_string(),
        border: styles.graph_download_border,
        Graph {
          datasets: [download_hist()],
          graph_colors: [*styles.graph_download_fill_color],
        }
      }
      rect {
        width: "flex(1)",
        height: styles.graph_height.to_string(),
        border: styles.graph_upload_border,
        Graph {
          datasets: [upload_hist()],
          graph_colors: [*styles.graph_upload_fill_color],
        }
      }
    }
  )
}

const NETWORK_DECIMAL_PLACES: usize = 2usize;

#[allow(non_snake_case)]
#[component]
pub fn NetworkComponent() -> Element {
  let config = use_context::<NetworkConfig>();
  let styles = use_context::<NetworkStyles>();
  let global_styles = use_context::<GlobalStyles>();

  let mut networks = Networks::new();

  let mut data = use_signal(NetworkData::default);

  let hist_size = ((global_styles.container_width - styles.graph_h_gap) / 2.) as usize;
  let mut download_hist = use_signal(|| CircularQueue::with_capacity(hist_size));
  let mut upload_hist = use_signal(|| CircularQueue::with_capacity(hist_size));

  let mut public_ip_str = use_signal(|| "".to_string());
  let mut local_ips_str = use_signal(|| "".to_string());

  use_hook(move || {
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
              .filter_map(|ip| ip.is_ipv4().then_some(ip.to_string()))
              .join(" ")
              .to_string(),
          );

          data.set(cur_data);
        }
        tokio::time::sleep(std::time::Duration::from_secs(config.update_interval)).await;
      }
    });
    if let Some(interval) = config.public_ip_retry_timeout {
      spawn(async move {
        loop {
          if let Some(ip) = public_ip::addr_with(GOOGLE_V6, public_ip::Version::V6).await {
            public_ip_str.set(ip.to_string());
          }
          tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
      });
    }
  });

  let value_color = styles.value_color;
  rsx!(
    if data().local_ips.is_empty() {
      label { color: styles.name_color.clone(), "Disconnected" },
    } else {
      rect {
        width: "100%",
        direction: "horizontal",
        content: "flex",
        label { color: styles.name_color.clone(), "WAN IP" },
        CursorArea {
          icon: CursorIcon::Copy,
          label {
            width: "flex(1)",
            text_align: "right",
            color: value_color.clone(),
            onclick: move |_| { Clipboard::new().unwrap().set_text(public_ip_str()).unwrap() },
            "{public_ip_str()}"
          },
        }
      }
      rect {
        width: "100%",
        direction: "horizontal",
        content: "flex",
        label { color: styles.name_color.clone(), "{data().network_name} IP" },
        CursorArea {
          icon: CursorIcon::Copy,
          label {
            width: "flex(1)",
            text_align: "right",
            color: value_color.clone(),
            onclick: move |_| {
              let local_ips_str = data().local_ips.iter().map(|ip| ip.to_string()).join("\n").to_string();
              Clipboard::new().unwrap().set_text(local_ips_str).unwrap();
            },
            "{local_ips_str()}"
          },
        }
      }
      rect {
        width: "100%",
        direction: "horizontal",
        content: "flex",
        spacing: global_styles.h_gap.to_string(),
        rect {
          width: "flex(1)",
          direction: "horizontal",
          label { color: styles.name_color.clone(), "Net Down" },
          LabelRight { color: value_color.clone(), "{format_speed(data().download_speed, NETWORK_DECIMAL_PLACES)}" },
        }
        rect {
          width: "flex(1)",
          direction: "horizontal",
          label { color: styles.name_color.clone(), "Net Up" },
          LabelRight { color: value_color.clone(), "{format_speed(data().upload_speed, NETWORK_DECIMAL_PLACES)}" },
        }
      }
      rect {
        width: "100%",
        direction: "horizontal",
        content: "flex",
        spacing: global_styles.h_gap.to_string(),
        rect {
          width: "flex(1)",
          direction: "horizontal",
          label { color: styles.name_color.clone(), "Total Down" },
          LabelRight { color: value_color.clone(), "{format_size(data().total_received, NETWORK_DECIMAL_PLACES)}" },
        }
        rect {
          width: "flex(1)",
          direction: "horizontal",
          label { color: styles.name_color.clone(), "Total Up" },
          LabelRight { color: value_color.clone(), "{format_size(data().total_transmitted, NETWORK_DECIMAL_PLACES)}" },
        }
      }
      NetworkGraphsComponent {
        download_hist: download_hist(),
        upload_hist: upload_hist(),
      }
    }
  )
}
