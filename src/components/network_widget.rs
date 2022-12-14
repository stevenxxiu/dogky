use gtk::gdk::RGBA;
use gtk::glib::{MainContext, Sender, PRIORITY_DEFAULT};
use gtk::prelude::{BoxExt, DrawingAreaExt, WidgetExt};
use gtk::{glib, Builder, DrawingArea, Label};
use public_ip::dns::GOOGLE_V6;
use std::net::IpAddr;
use std::sync::Arc;
use sysinfo::{NetworkData, NetworkExt, RefreshKind, System, SystemExt};

use crate::config::{NetworkGraphContainerProps, NetworkProps};
use crate::custom_components::build_graph;
use crate::format_size::{format_size, format_speed};
use crate::gtk_utils::{set_copyable_label, set_label};
use crate::serde_structs::SerializableRegex;
use crate::utils::join_str_iter;

const NETWORK_DECIMAL_PLACES: usize = 2usize;

#[derive(Clone)]
pub struct NetworkWidget {
  builder: Arc<Builder>,
  download_graph_sender: Sender<f32>,
  upload_graph_sender: Sender<f32>,
}

impl NetworkWidget {
  pub fn build(props: NetworkProps, container_width: u32) -> gtk::Box {
    let builder = Arc::new(Builder::from_resource("/org/dogky/network_widget.ui"));
    let container: gtk::Box = builder.object("network_widget").unwrap();

    let props = Arc::new(props);
    let refresh_kind = RefreshKind::new().with_networks().with_networks_list();
    let system = System::new_with_specifics(refresh_kind);

    let [download_graph_sender, upload_graph_sender] =
      NetworkWidget::build_graphs(&props.graphs, container_width, &builder);

    let updater = NetworkWidget {
      builder: builder.clone(),
      download_graph_sender,
      upload_graph_sender,
    };
    updater.update(Arc::new(system), props.clone());

    NetworkWidgetPublicIp::build(props.public_ip_update_interval, builder.clone());

    container
  }

  fn build_graphs(props: &NetworkGraphContainerProps, container_width: u32, builder: &Builder) -> [Sender<f32>; 2] {
    let container = builder.object::<gtk::Box>("network_usage_graph_container").unwrap();
    let graph_specs = [
      (&props.download, props.download.maximum_bytes_per_sec),
      (&props.upload, props.upload.maximum_bytes_per_sec),
    ];
    let margin = (container_width - graph_specs.len() as u32 * props.width) / (graph_specs.len() as u32 - 1);
    container.set_spacing(margin as i32);
    graph_specs.map(|(graph_props, max_value)| {
      let graph = DrawingArea::new();
      graph.set_content_width(props.width as i32);
      graph.set_content_height(props.height as i32);
      container.append(&graph);

      let border_color = RGBA::parse(&graph_props.border_color).unwrap();
      let fill_color = RGBA::parse(&graph_props.fill_color).unwrap();
      let (sender, receiver) = MainContext::channel(PRIORITY_DEFAULT);
      build_graph(graph, receiver, max_value as f32, &border_color, &fill_color);
      sender
    })
  }

  fn update_is_connected(&self, is_connected: bool) {
    self
      .builder
      .object::<gtk::Box>("network_connected_container")
      .unwrap()
      .set_visible(is_connected);
    self
      .builder
      .object::<Label>("network_error_label")
      .unwrap()
      .set_visible(!is_connected);
  }

  fn get_network<'a>(
    system: &'a System,
    interface_regex: &'a SerializableRegex,
  ) -> Option<(&'a String, &'a NetworkData)> {
    system
      .networks()
      .into_iter()
      .filter(|(interface_name, _data)| interface_regex.is_match(interface_name))
      .next()
  }

  /// Collects into a vector, so we can test if there's no IP, and to print them
  fn get_local_ips(network_name: &str) -> Vec<IpAddr> {
    local_ip_address::list_afinet_netifas()
      .unwrap()
      .into_iter()
      .filter_map(|(name, ip)| (name == network_name).then_some(ip))
      .collect()
  }

  fn update_local_ips(&self, network_with_data: &Option<(&String, &NetworkData)>) -> bool {
    if network_with_data.is_none() {
      return false;
    }
    let (network_name, _network_data) = network_with_data.unwrap();
    let local_ips = NetworkWidget::get_local_ips(network_name);
    if local_ips.is_empty() {
      return false;
    }
    set_label(&self.builder, "network_interface", network_name);

    // Only include IPv4, as IPv6 addresses are too long
    let local_ips_str = join_str_iter(
      local_ips
        .into_iter()
        .filter_map(|ip| ip.is_ipv4().then(|| ip.to_string())),
      " ",
    );
    set_copyable_label(&self.builder, "local_ips", local_ips_str);
    true
  }

  fn update_network(&self, network_data: &NetworkData, update_interval: u32) {
    let total_received = network_data.total_received();
    let download_total_str = format_size(total_received, NETWORK_DECIMAL_PLACES);
    set_label(&self.builder, "download_total", &download_total_str);

    let total_transmitted = network_data.total_transmitted();
    let total_transmitted_str = format_size(total_transmitted, NETWORK_DECIMAL_PLACES);
    set_label(&self.builder, "upload_total", &total_transmitted_str);

    let download_speed = network_data.received() as f32 / update_interval as f32;
    let download_speed_str = format_speed(download_speed, NETWORK_DECIMAL_PLACES);
    set_label(&self.builder, "download_speed", &download_speed_str);
    self.download_graph_sender.send(download_speed).unwrap();

    let upload_speed = network_data.transmitted() as f32 / update_interval as f32;
    let upload_speed_str = format_speed(upload_speed, NETWORK_DECIMAL_PLACES);
    set_label(&self.builder, "upload_speed", &upload_speed_str);
    self.upload_graph_sender.send(upload_speed).unwrap();
  }

  fn update(&self, mut system: Arc<System>, props: Arc<NetworkProps>) {
    let system_deref = Arc::get_mut(&mut system).unwrap();
    system_deref.refresh_networks();

    let network_with_data = NetworkWidget::get_network(system_deref, &props.interface_regex);
    let is_connected = self.update_local_ips(&network_with_data);
    self.update_is_connected(is_connected);
    if let Some((_network_name, network_data)) = network_with_data {
      self.update_network(network_data, props.update_interval);
    }

    let self_clone = self.clone();
    glib::source::timeout_add_seconds_local_once(props.update_interval, move || self_clone.update(system, props));
  }
}

pub struct NetworkWidgetPublicIp {
  runtime: Arc<tokio::runtime::Runtime>,
  builder: Arc<Builder>,
}

impl NetworkWidgetPublicIp {
  fn build(update_interval: Option<u32>, builder: Arc<Builder>) {
    if update_interval.is_none() {
      builder
        .object::<gtk::Box>("wan_ip_container")
        .unwrap()
        .set_visible(false);
      return;
    }
    let runtime = Arc::new(
      tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap(),
    );
    let public_ip_updater = NetworkWidgetPublicIp {
      runtime: runtime.clone(),
      builder,
    };
    public_ip_updater.update(update_interval.unwrap());
  }

  fn update(self, update_interval: u32) {
    // Use IPv6 for public IP
    if let Some(cur_public_ip) = self
      .runtime
      .block_on(public_ip::addr_with(GOOGLE_V6, public_ip::Version::V6))
    {
      set_copyable_label(&self.builder, "wan_ip", cur_public_ip.to_string());
    }
    glib::source::timeout_add_seconds_local_once(update_interval, move || self.update(update_interval));
  }
}
