use chrono::NaiveDateTime;
use gtk::prelude::{GestureExt, WidgetExt};
use gtk::{glib, Builder};
use heck::ToTitleCase;
use std::fs::File;
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

pub struct WeatherWidget {
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

impl WeatherWidget {
  pub fn build(props: WeatherProps) -> gtk::Box {
    let builder = Builder::from_resource("/org/dogky/weather_widget.ui");
    let container: gtk::Box = builder.object("weather_widget").unwrap();

    let props = Arc::new(props);
    let cache_path = Arc::new(get_xdg_dirs().place_cache_file("weather.json").unwrap());
    let updater = WeatherWidget {
      cache_path,
      data: Arc::new(None),
      error_str: Arc::new(None),
    };
    updater.update(props.clone(), &builder);
    add_click_listener(props.clone(), &container);
    container
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

  fn update_components(self, builder: &Builder) -> Self {
    let error_container: gtk::Box = builder.object("error_container").unwrap();
    match Option::as_ref(&self.error_str) {
      Some(error_text) => {
        error_container.set_visible(true);
        set_label(builder, "error", &error_text);
        return self;
      }
      None => error_container.set_visible(false),
    }
    let data = Option::as_ref(&self.data).unwrap();
    let icon_key: String = data.weather[0].icon.chars().take(2).collect();
    set_label(builder, "icon", *ICON_MAP.get(icon_key.as_str()).unwrap());
    set_label(builder, "conditions", &data.weather[0].description.to_title_case());
    set_label(builder, "temperature", &format!("{}°C", data.main.temp.round()));
    set_label(builder, "humidity", &format!("{}%", data.main.humidity));
    let wind = format!(
      "{} kph {}",
      data.wind.speed.round(),
      degrees_to_direction(data.wind.deg)
    );
    set_label(builder, "wind", &wind);
    set_label(builder, "sunrise", &format_sun_timestamp(data.sys.sunrise));
    set_label(builder, "sunset", &format_sun_timestamp(data.sys.sunset));
    self
  }

  fn update(mut self, props: Arc<WeatherProps>, builder: &Builder) {
    self = self.update_data(props.clone());
    let timeout = if Option::as_ref(&self.error_str).is_none() {
      props.update_interval
    } else {
      props.retry_timeout
    };
    self = self.update_components(builder);
    glib::source::timeout_add_seconds_local_once(
      timeout,
      glib::clone!(@weak builder => move || {
        self.update(props, &builder);
      }),
    );
  }
}
