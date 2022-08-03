use chrono::NaiveDateTime;
use gtk::glib;
use gtk::prelude::{GestureExt, ObjectExt, WidgetExt};
use heck::ToTitleCase;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use phf::phf_map;

use crate::api::{get_weather, WeatherData};
use crate::components::WeatherWidget;
use crate::config::WeatherProps;
use crate::path::get_xdg_dirs;

// Unicode weather symbols to use
static ICON_MAP: phf::Map<&'static str, &'static str> = phf_map! {
  "01" => "☀",
  "02" => "🌤",
  "03" => "🌥",
  "04" => "☁",
  "09" => "🌧",
  "10" => "🌦",
  "11" => "🌩",
  "13" => "🌨",
  "50" => "🌫",
};

fn add_click_listener(props: Arc<WeatherProps>, weather_widget: &WeatherWidget) {
  let gesture = gtk::GestureClick::new();
  gesture.connect_released(glib::clone!(@strong props => move |gesture, _, _, _| {
    gesture.set_state(gtk::EventSequenceState::Claimed);
    // Open weather forecast link
    open::that(format!("https://openweathermap.org/city/{0}#weather-widget", props.openweather_city_id)).unwrap();
  }));
  weather_widget.add_controller(&gesture);
  weather_widget.set_cursor_from_name(Option::from("hand"));
}

pub struct WeatherWidgetUpdater {
  cache_path: Arc<PathBuf>,
  data: Arc<Option<WeatherData>>,
  error_str: Arc<Option<String>>,
}

fn degrees_to_direction(degrees: f64) -> &'static str {
  let directions = [
    "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW", "NW", "NNW",
  ];
  let sector_angle = 360.0 / (directions.len() as f64);
  // `[0]` is `-sector_angle / 2 <= degrees < sector_angle / 2`
  // `[1]` is `sector_angle / 2 <= degrees < 3 * sector_angle / 2`
  // ...
  let i = ((degrees + sector_angle / 2.0) / sector_angle) as usize % directions.len();
  directions[i]
}

fn format_sun_timestamp(timestamp: u64) -> String {
  let naive = NaiveDateTime::from_timestamp(timestamp as i64, 0);
  naive.format("%l:%M %p").to_string()
}

impl WeatherWidgetUpdater {
  pub fn init(props: WeatherProps, weather_widget: &WeatherWidget) {
    let props = Arc::new(props);
    add_click_listener(props.clone(), weather_widget);
    let cache_path = Arc::new(get_xdg_dirs().place_cache_file("weather.json").unwrap());
    let updater = WeatherWidgetUpdater {
      cache_path,
      data: Arc::new(None),
      error_str: Arc::new(None),
    };
    updater.update(props.clone(), weather_widget);
  }

  fn load_cache(&mut self) {
    let data_file = File::open(self.cache_path.as_ref()).unwrap();
    let data = serde_json::from_reader(data_file).unwrap();
    self.data = Arc::new(Some(data));
  }

  fn update_data(mut self, props: Arc<WeatherProps>) -> Self {
    // No need to fetch data from server if cache time is close enough
    if let Ok(metadata) = std::fs::metadata(self.cache_path.as_ref()) {
      let cache_time = metadata.modified().unwrap();
      let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
      if time_since_cache < Duration::from_secs(props.update_interval as u64) {
        self.load_cache();
        return self;
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
    self
  }

  fn update_components(self, weather_widget: &WeatherWidget) -> Self {
    weather_widget.set_property("error", self.error_str.as_ref());
    if self.error_str.is_some() {
      return self;
    }
    let data = Option::as_ref(&self.data).unwrap();
    let icon_key: String = data.weather[0].icon.chars().take(2).collect();
    weather_widget.set_property("icon", *ICON_MAP.get(icon_key.as_str()).unwrap());
    weather_widget.set_property("conditions", data.weather[0].description.to_title_case());
    weather_widget.set_property("temperature", format!("{}°C", data.main.temp.round()));
    weather_widget.set_property("humidity", format!("{}%", data.main.humidity));
    let wind = format!(
      "{} kph {}",
      data.wind.speed.round(),
      degrees_to_direction(data.wind.deg)
    );
    weather_widget.set_property("wind", wind);
    weather_widget.set_property("sunrise", format_sun_timestamp(data.sys.sunrise));
    weather_widget.set_property("sunset", format_sun_timestamp(data.sys.sunset));
    self
  }

  fn update(mut self, props: Arc<WeatherProps>, weather_widget: &WeatherWidget) {
    self = self.update_data(props.clone());
    let timeout = if Option::as_ref(&self.error_str).is_none() {
      props.update_interval
    } else {
      props.retry_timeout
    };
    self = self.update_components(weather_widget);
    glib::source::timeout_add_seconds_local_once(
      timeout,
      glib::clone!(@weak weather_widget => move || {
        self.update(props, &weather_widget);
      }),
    );
  }
}
