use freya::prelude::*;
use styles_config::{GlobalStyles, StylesConfig};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::WindowLevel;
use xcb::{x, Xid, XidNew};

use components::{CpuMemoryComponent, MachineInfoComponent, WeatherComponent};
use custom_components::Separator;

mod api;
mod components;
mod config;
mod custom_components;
mod format_size;
mod path;
mod serde_structs;
mod styles_config;
mod ui_utils;
mod utils;

fn set_wm_states(window_id: u32) {
  let (conn, _screen_num) = xcb::Connection::connect(None).unwrap();
  let setup = conn.get_setup();
  let screen = setup.roots().next().unwrap(); // Assume screen 0
  let window: x::Window = unsafe { XidNew::new(window_id) };

  let get_net_wm_atom = |name| {
    let cookie = conn.send_request(&x::InternAtom {
      only_if_exists: true,
      name,
    });
    conn.wait_for_reply(cookie).unwrap().atom()
  };

  let wm_state = get_net_wm_atom(b"_NET_WM_STATE");
  for name in [
    b"_NET_WM_STATE_STICKY".as_ref(),
    b"_NET_WM_STATE_SKIP_TASKBAR".as_ref(),
    b"_NET_WM_STATE_SKIP_PAGER".as_ref(),
    b"_NET_WM_STATE_BELOW".as_ref(),
  ] {
    let atom = get_net_wm_atom(name);
    let data = x::ClientMessageData::Data32([1, atom.resource_id(), 0, 0, 0]);
    let event = x::ClientMessageEvent::new(window, wm_state, data);
    let cookie = conn.send_request_checked(&x::SendEvent {
      propagate: false,
      destination: x::SendEventDest::Window(screen.root()),
      event_mask: x::EventMask::STRUCTURE_NOTIFY,
      event: &event,
    });
    conn.check_request(cookie).unwrap();
  }
}

fn app() -> Element {
  let styles = consume_context::<StylesConfig>();
  let padding_parsed = ui_utils::parse_padding(&styles.padding).unwrap();
  let global_styles = GlobalStyles {
    container_width: styles.width as f32 - padding_parsed.left() - padding_parsed.right(),
    padding: styles.padding.clone(),
    h_gap: styles.h_gap,
  };
  use_context_provider(|| global_styles);

  let config = config::load_config().unwrap();

  rsx!(rect {
    width: "100%",
    height: "100%",
    direction: "vertical",
    background: styles.background_color,
    color: styles.text_color,
    font_size: styles.text_size.to_string(),
    padding: styles.padding.clone(),
    WeatherComponent { config: config.weather, styles: styles.weather },
    Separator {},
    MachineInfoComponent { styles: styles.machine_info },
    Separator {},
    CpuMemoryComponent { config: config.cpu_memory, styles: styles.cpu_memory  }
  })
}

pub fn main() {
  let styles = styles_config::load_config().unwrap();
  let width = styles.width;
  let font = styles.font.clone();
  launch_cfg(
    app,
    LaunchConfig::<StylesConfig> {
      state: Some(styles),
      default_fonts: vec![font],
      ..Default::default()
    }
    .on_setup(move |window| {
      let monitor = window.current_monitor().unwrap();
      window.set_outer_position(LogicalPosition::new(monitor.size().width - width, 0));
      let _ = window.request_inner_size(LogicalSize::new(width, monitor.size().height));
      match window.window_handle().unwrap().as_raw() {
        RawWindowHandle::Xlib(window) => set_wm_states(window.window as u32),
        _ => panic!("Not on X11"),
      }
    })
    .with_background("transparent")
    .with_window_attributes(|attributes| {
      attributes
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnBottom)
    }),
  )
}
