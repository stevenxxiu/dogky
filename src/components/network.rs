use circular_queue::CircularQueue;
use iced::alignment::Horizontal;
use iced::mouse::Interaction;
use iced::widget::{canvas, column, container, mouse_area, row, text, Row};
use iced::{clipboard, time, Element, Length, Subscription, Task};
use public_ip::dns::GOOGLE_V6;
use std::net::IpAddr;
use std::time::Duration;
use sysinfo::Networks;

use crate::config::NetworkProps;
use crate::custom_components::Graph;
use crate::format_size::{format_size, format_speed};
use crate::message::Message;
use crate::styles::network as styles;
use crate::ui_utils::{expand_right, space_row};
use crate::utils::join_str_iter;

pub struct NetworkComponent {
  config_props: NetworkProps,
  container_width: f32,
  networks: Networks,
  live: NetworkLiveProps,
  history: HistoryProps,
}

#[derive(Default)]
struct NetworkLiveProps {
  network_name: String,
  local_ips: Vec<IpAddr>,
  public_ip: Option<IpAddr>,
  total_received: u64,
  total_transmitted: u64,
  upload_speed: f32,
  download_speed: f32,
}

struct HistoryProps {
  upload_speed: CircularQueue<f32>,
  download_speed: CircularQueue<f32>,
}

const NETWORK_DECIMAL_PLACES: usize = 2usize;

impl NetworkComponent {
  pub fn new(config_props: NetworkProps, container_width: f32) -> Self {
    let networks = Networks::new();

    let process_data_size = ((container_width - styles::GRAPH_H_GAP) / 2.0) as usize;

    let mut res = Self {
      config_props,
      container_width,
      networks,
      live: NetworkLiveProps::default(),
      history: HistoryProps {
        upload_speed: CircularQueue::with_capacity(process_data_size),
        download_speed: CircularQueue::with_capacity(process_data_size),
      },
    };
    let _ = res.update(Message::NetworkWanIPTick);
    res
  }

  /// Collect into a vector, so we can test if there's no IP, or to print them
  fn get_local_ips(network_name: &str) -> Vec<IpAddr> {
    local_ip_address::list_afinet_netifas()
      .unwrap()
      .into_iter()
      .filter_map(|(name, ip)| (name == network_name).then_some(ip))
      .collect()
  }

  fn update_data(&mut self) {
    self.networks.refresh(true);
    let network_with_data = self
      .networks
      .into_iter()
      .find(|(interface_name, _data)| self.config_props.interface_regex.is_match(interface_name));

    if network_with_data.is_none() {
      return;
    }
    let (network_name, network_data) = network_with_data.unwrap();

    self.live.local_ips = NetworkComponent::get_local_ips(network_name);
    if self.live.local_ips.is_empty() {
      return;
    }
    self.live.network_name = network_name.to_string();

    let props = &self.config_props;
    let graph_config = &self.config_props.graphs;

    self.live.total_received = network_data.total_received();
    self.live.total_transmitted = network_data.total_transmitted();

    self.live.download_speed = network_data.received() as f32 / props.update_interval as f32;
    self
      .history
      .download_speed
      .push(self.live.download_speed / graph_config.download.maximum_bytes_per_sec as f32);

    self.live.upload_speed = network_data.transmitted() as f32 / props.update_interval as f32;
    self
      .history
      .upload_speed
      .push(self.live.upload_speed / graph_config.upload.maximum_bytes_per_sec as f32);
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::NetworkTick => {
        self.update_data();
        Task::none()
      }
      Message::NetworkWanIPTick => Task::perform(public_ip::addr_with(GOOGLE_V6, public_ip::Version::V6), |res| {
        Message::NetworkWanIPAssign(res)
      }),
      Message::NetworkWanIPAssign(res) => {
        self.live.public_ip = res;
        Task::none()
      }
      Message::NetworkWanIPClick => {
        if let Some(public_ip) = self.live.public_ip {
          clipboard::write(public_ip.to_string())
        } else {
          Task::none()
        }
      }
      Message::NetworkLocalIPClick => {
        let local_ips_str = join_str_iter(self.live.local_ips.iter().map(|ip| ip.to_string()), "\n");
        clipboard::write(local_ips_str)
      }
      _ => Task::none(),
    }
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let props = &self.config_props;
    let mut subscriptions =
      vec![time::every(Duration::from_secs(props.update_interval)).map(|_instant| Message::NetworkTick)];
    if let Some(interval) = props.public_ip_update_interval {
      if self.live.public_ip.is_none() {
        subscriptions.push(time::every(Duration::from_secs(interval)).map(|_instant| Message::NetworkWanIPTick));
      }
    }
    Subscription::batch(subscriptions)
  }

  fn view_graphs(&self) -> Row<Message> {
    let graph_width: f32 = (self.container_width - styles::GRAPH_H_GAP) / 2.0;
    row![
      container(
        canvas(Graph {
          datasets: vec![self.history.download_speed.clone()],
          width: graph_width,
          height: styles::GRAPH_HEIGHT,
          border_color: styles::GRAPH_DOWNLOAD_BORDER_COLOR,
          graph_colors: vec![styles::GRAPH_DOWNLOAD_FILL_COLOR],
          cache: canvas::Cache::new(),
        })
        .width(graph_width)
        .height(styles::GRAPH_HEIGHT)
      ),
      container(
        canvas(Graph {
          datasets: vec![self.history.upload_speed.clone()],
          width: graph_width,
          height: styles::GRAPH_HEIGHT,
          border_color: styles::GRAPH_UPLOAD_BORDER_COLOR,
          graph_colors: vec![styles::GRAPH_UPLOAD_FILL_COLOR],
          cache: canvas::Cache::new(),
        })
        .width(graph_width)
        .height(styles::GRAPH_HEIGHT)
      )
    ]
    .width(Length::Fill)
    .spacing(styles::GRAPH_H_GAP)
  }

  pub fn view(&self) -> Element<Message> {
    macro_rules! name_text {
      ($s:expr) => {
        text($s).color(styles::NAME_COLOR)
      };
    }
    macro_rules! value_text {
      ($s:expr) => {
        text($s).color(styles::VALUE_COLOR)
      };
    }

    let live = &self.live;

    if live.local_ips.is_empty() {
      name_text!("Disconnected").align_x(Horizontal::Left).into()
    } else {
      let wan_ip_text = space_row![row![
        name_text!("WAN IP"),
        expand_right![value_text!(live
          .public_ip
          .map(|ip| ip.to_string())
          .unwrap_or("".to_string()))],
      ]]
      .width(Length::Fill);
      let wan_ip_copy = mouse_area(wan_ip_text)
        .interaction(Interaction::Copy)
        .on_press(Message::NetworkWanIPClick);

      // Only include IPv4, as IPv6 addresses are too long
      let local_ips_str = join_str_iter(
        self
          .live
          .local_ips
          .iter()
          .filter(|ip| ip.is_ipv4())
          .map(|ip| ip.to_string()),
        " ",
      );
      let local_ips_text = space_row![row![
        name_text!(format!("{} IP", live.network_name)),
        expand_right![value_text!(local_ips_str)],
      ]]
      .width(Length::Fill);
      let local_ips_copy = mouse_area(local_ips_text)
        .interaction(Interaction::Copy)
        .on_press(Message::NetworkLocalIPClick);

      column![
        wan_ip_copy,
        local_ips_copy,
        space_row![row![
          row![
            name_text!("Net Down"),
            expand_right![value_text!(format_speed(live.download_speed, NETWORK_DECIMAL_PLACES))]
          ]
          .width(Length::Fill),
          row![
            name_text!("Net Up"),
            expand_right![value_text!(format_speed(live.upload_speed, NETWORK_DECIMAL_PLACES))]
          ]
          .width(Length::Fill),
        ]],
        space_row![row![
          row![
            name_text!("Total Down"),
            expand_right![value_text!(format_size(live.total_received, NETWORK_DECIMAL_PLACES))]
          ]
          .width(Length::Fill),
          row![
            name_text!("Total Up"),
            expand_right![value_text!(format_size(live.total_transmitted, NETWORK_DECIMAL_PLACES))]
          ]
          .width(Length::Fill),
        ]],
        self.view_graphs(),
      ]
      .width(Length::Fill)
      .into()
    }
  }
}
