use chrono::{FixedOffset, NaiveDateTime};
use gtk::gdk::Display;
use gtk::prelude::{GestureExt, WidgetExt};
use gtk::{glib, Builder, CssProvider, StyleContext};
use heck::ToTitleCase;
use std::fs::File;
use std::ops::Add;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use phf::phf_map;

use crate::api::{get_weather, WeatherData};
use crate::config::WeatherProps;
use crate::gtk_utils::set_label;
use crate::path::get_xdg_dirs;

// Unicode weather symbols to use
static ICON_MAP: phf::Map<&'static str, &'static str> = phf_map! {
  "01" => "☀️",
  "02" => "🌤️",
  "03" => "🌥️",
  "04" => "☁️",
  "09" => "🌧️",
  "10" => "🌦️",
  "11" => "🌩️",
  "13" => "🌨️",
  "50" => "🌫️",
};

#[derive(Clone)]
pub struct WeatherWidget {
  builder: Arc<Builder>,
  cache_path: Arc<PathBuf>,
  data: Arc<Option<WeatherData>>,
  error_str: Arc<Option<String>>,
  css_provider: Arc<CssProvider>,
}

fn format_sun_timestamp(timestamp: u64, timezone: FixedOffset) -> String {
  let date_time = NaiveDateTime::from_timestamp(timestamp as i64, 0).add(timezone);
  date_time.format("%-l:%M %p").to_string()
}

impl WeatherWidget {
  pub fn build(props: WeatherProps) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/weather_widget.ui");
    let container: gtk::Box = builder.object("weather_widget").unwrap();

    let props = Arc::new(props);
    let cache_path = Arc::new(get_xdg_dirs().place_cache_file("weather.json").unwrap());
    let mut updater = WeatherWidget {
      builder: Arc::new(builder),
      cache_path,
      data: Arc::new(None),
      error_str: Arc::new(None),
      css_provider: Arc::new(CssProvider::new()),
    };
    let display = Display::default().expect("Could not connect to a display.");
    let priority = gtk::STYLE_PROVIDER_PRIORITY_APPLICATION;
    StyleContext::add_provider_for_display(&display, updater.css_provider.as_ref(), priority);
    updater.update(props.clone());
    WeatherWidget::add_click_listener(props.clone(), &container);
    container
  }

  fn add_click_listener(props: Arc<WeatherProps>, container: &gtk::Box) {
    let gesture = gtk::GestureClick::new();
    gesture.connect_released(glib::clone!(@strong props => move |gesture, _, _, _| {
      gesture.set_state(gtk::EventSequenceState::Claimed);
      // Open weather forecast link
      open::that(format!("https://openweathermap.org/city/{0}#weather-widget", props.openweather_city_id)).unwrap();
    }));
    container.add_controller(&gesture);
    container.set_cursor_from_name(Option::from("pointer"));
  }

  fn load_cache(&mut self) {
    let data_file = File::open(self.cache_path.as_ref()).unwrap();
    let data = serde_json::from_reader(data_file).unwrap();
    self.data = Arc::new(Some(data));
  }

  fn update_error(&mut self) {
    self
      .builder
      .object::<gtk::Box>("weather_error_container")
      .unwrap()
      .set_visible(self.error_str.is_some());
    self
      .builder
      .object::<gtk::Box>("weather_connected_container")
      .unwrap()
      .set_visible(self.error_str.is_none());
    if let Some(error_str) = self.error_str.as_ref() {
      set_label(self.builder.as_ref(), "error", &error_str);
    }
  }

  fn update_data(&mut self, props: &WeatherProps) {
    // No need to fetch data from server if cache time is close enough
    if let Ok(metadata) = std::fs::metadata(self.cache_path.as_ref()) {
      let cache_time = metadata.modified().unwrap();
      let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
      if time_since_cache < Duration::from_secs(props.update_interval as u64) {
        self.load_cache();
      }
    }

    // Update data from server
    match get_weather(props.openweather_city_id, &props.openweather_api_key) {
      Ok(weather_data) => {
        let data_file = File::create(self.cache_path.as_ref()).unwrap();
        serde_json::to_writer(data_file, &weather_data).unwrap();
        self.data = Arc::new(Some(weather_data));
        self.error_str = Arc::new(None);
      }
      Err(error) => {
        self.error_str = Arc::new(Some(error.to_string()));
      }
    }
  }

  fn update_components(&mut self) {
    self.update_error();
    if self.error_str.is_some() {
      return;
    }
    let builder = self.builder.as_ref();
    let data = Option::as_ref(&self.data).unwrap();

    let icon_key: String = data.weather[0].icon.chars().take(2).collect();
    set_label(builder, "icon", *ICON_MAP.get(icon_key.as_str()).unwrap());

    set_label(builder, "conditions", &data.weather[0].description.to_title_case());
    set_label(builder, "temperature", &format!("{:.0}°C", data.main.temp));
    set_label(builder, "humidity", &format!("{}%", data.main.humidity));

    let wind_speed = format!("{:.1} m/s", data.wind.speed);
    set_label(builder, "wind_speed", &wind_speed);

    // The data is offset by -180°. The wind degrees character used is `⮕`, which is at 90°.
    let wind_css = format!(
      "#weather-wind-direction {{ transform: rotate({}deg); }}",
      data.wind.deg + 180.0 - 90.0
    );
    self.css_provider.load_from_data(wind_css.as_bytes());

    let timezone = FixedOffset::east(data.timezone);
    set_label(builder, "sunrise", &format_sun_timestamp(data.sys.sunrise, timezone));
    set_label(builder, "sunset", &format_sun_timestamp(data.sys.sunset, timezone));
  }

  fn update(&mut self, props: Arc<WeatherProps>) {
    self.update_data(&props.as_ref());
    let timeout = if self.error_str.is_none() {
      props.update_interval
    } else {
      props.retry_timeout
    };
    self.update_components();

    let mut self_clone = self.clone();
    glib::source::timeout_add_seconds_local_once(timeout, move || self_clone.update(props));
  }
}
