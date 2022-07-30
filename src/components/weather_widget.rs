use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use gtk::glib;
use gtk::glib::clone;

use crate::api::{get_weather, WeatherData};
use crate::config::WeatherProps;
use crate::path::get_xdg_dirs;

#[derive(Clone)]
pub struct WeatherWidget {
  cache_path: PathBuf,
  data: Option<WeatherData>,
  error: Option<String>,
}

impl WeatherWidget {
  pub fn create(props: &WeatherProps, container: &gtk::Box) -> () {
    let cache_path = get_xdg_dirs().place_cache_file("weather.json").unwrap();
    let mut widget = WeatherWidget {
      cache_path,
      data: None,
      error: None,
    };
    widget.update(&props, &container);
  }

  fn load_cache(&mut self) -> Result<(), std::io::Error> {
    let data_file = File::open(&self.cache_path)?;
    let data = serde_json::from_reader(data_file).unwrap();
    self.data = Some(data);
    Ok(())
  }

  fn update_data(&mut self, props: &WeatherProps) -> Result<(), std::io::Error> {
    // No need to fetch data from server if cache time is close enough
    if let Ok(cache_time) = fs::metadata(&self.cache_path)?.modified() {
      let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
      if time_since_cache < Duration::from_secs(props.update_interval as u64) {
        self.load_cache()?;
        return Ok(());
      }
    }

    // Update data from server
    match get_weather(props.openweather_city_id, &props.openweather_api_key) {
      Ok(weather_data) => {
        let data_file = File::create(&self.cache_path)?;
        serde_json::to_writer(data_file, &weather_data)?;
        self.data = Some(weather_data);
        self.error = None;
      }
      Err(error) => {
        self.error = Some(error.to_string());
      }
    }
    Ok(())
  }

  fn update_components(self: &Self, container: &gtk::Box) {}

  fn update(&mut self, props: &WeatherProps, container: &gtk::Box) {
    let timeout = if self.update_data(props).is_ok() {
      props.update_interval
    } else {
      props.retry_timeout
    };
    self.update_components(container);
    glib::source::timeout_add_seconds_local_once(
      timeout,
      clone!(@strong self as self_, @strong props, @weak container => move || {
        self_.clone().update(&props, &container);
      }),
    );
  }
}
