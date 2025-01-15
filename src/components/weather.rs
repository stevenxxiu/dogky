use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, FixedOffset};
use freya::prelude::*;
use heck::ToTitleCase;

use phf::phf_map;

use crate::api::{get_weather, WeatherData};
use crate::config::WeatherConfig;
use crate::path::get_xdg_dirs;
use crate::styles_config::{GlobalStyles, WeatherStyles};

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

fn format_sun_timestamp(timestamp: u64, timezone: FixedOffset) -> String {
  let date_time = DateTime::from_timestamp(timestamp as i64, 0).unwrap() + timezone;
  date_time.format("%-l:%M %p").to_string()
}

fn load_cache(path: PathBuf) -> WeatherData {
  let data_file = File::open(&path).unwrap();
  serde_json::from_reader(data_file).unwrap()
}

fn update_data(config: &WeatherConfig, cache_path: &PathBuf) -> Result<WeatherData, String> {
  // No need to fetch data from server if cache time is close enough
  if let Ok(metadata) = std::fs::metadata(cache_path) {
    let cache_time = metadata.modified().unwrap();
    let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
    if time_since_cache < Duration::from_secs(config.update_interval) {
      return Ok(load_cache(cache_path.to_path_buf()));
    }
  }

  // Update data from server
  match get_weather(config.openweather_city_id, &config.openweather_api_key) {
    Ok(weather_data) => {
      let data_file = File::create(cache_path).unwrap();
      serde_json::to_writer(data_file, &weather_data).unwrap();
      Ok(weather_data)
    }
    Err(error) => Err(error.to_string()),
  }
}

#[allow(non_snake_case)]
#[component]
pub fn WeatherComponent(config: WeatherConfig, styles: WeatherStyles) -> Element {
  let global_styles = use_context::<GlobalStyles>();
  let cache_path = get_xdg_dirs().place_cache_file("weather.json").unwrap();

  let mut has_err = use_signal(|| false);
  let mut error_str = use_signal(|| "".into());

  let city_id = config.openweather_city_id;
  let mut cond_icon = use_signal(|| "");
  let mut conditions = use_signal(|| "".into());
  let mut temperature = use_signal(|| "".into());
  let mut humidity = use_signal(|| "".into());
  let mut wind_speed = use_signal(|| "".into());
  let mut wind_deg = use_signal(|| 0f32);
  let mut sunrise_text = use_signal(|| "".into());
  let mut sunset_text = use_signal(|| "".into());

  use_hook(move || {
    spawn(async move {
      loop {
        let timeout = match update_data(&config, &cache_path) {
          Ok(data) => {
            has_err.set(false);

            let cond_icon_key: String = data.weather[0].icon.chars().take(2).collect();
            cond_icon.set(*ICON_MAP.get(cond_icon_key.as_str()).unwrap());

            conditions.set(data.weather[0].description.to_title_case());
            temperature.set(format!("{:.0}°C", data.main.temp));
            humidity.set(format!("{}%", data.main.humidity));
            wind_speed.set(format!("{:.1} m/s", data.wind.speed));
            wind_deg.set(data.wind.deg);

            let timezone = FixedOffset::east_opt(data.timezone).unwrap();
            sunrise_text.set(format_sun_timestamp(data.sys.sunrise, timezone));
            sunset_text.set(format_sun_timestamp(data.sys.sunset, timezone));

            config.update_interval
          }
          Err(error_str_) => {
            has_err.set(true);
            error_str.set(error_str_);
            config.retry_timeout
          }
        };
        tokio::time::sleep(std::time::Duration::from_secs(timeout)).await;
      }
    })
  });

  rsx!(
    CursorArea {
      icon: CursorIcon::Pointer,
      rect {
        width: "100%",
        direction: "vertical",
        padding: styles.container_padding,
        onclick: move |_| {
          // Open weather forecast link
          open::that(format!("https://openweathermap.org/city/{0}#weather-widget", city_id))
          .unwrap();
        },
        if has_err() {
          rect {
            width: "100%",
            direction: "horizontal",
            spacing: global_styles.h_gap.to_string(),
            main_align: "center",
            label { "Weather:" },
            label { "{error_str}" },
          }
        } else {
          rect {
            width: "100%",
            direction: "horizontal",
            main_align: "center",
            cross_align: "center",
            spacing: global_styles.h_gap.to_string(),
            label {
              font_family: "Noto Color Emoji",
              font_size: styles.cond_icon_size.to_string(),
              "{cond_icon}",
            }
            label { "{conditions}" }
            label { color: styles.value_color.to_string(), "{temperature}" }
          }
          rect {
            width: "100%",
            direction: "horizontal",
            main_align: "center",
            spacing: global_styles.h_gap.to_string(),
            label { "Humidity" }, label { color: styles.value_color.to_string(), "{humidity}" }
            label { "Wind" }, label { color: styles.value_color.to_string(), "{wind_speed}" }
            label {
              margin: styles.wind_arrow_margin.to_string(),
              color: styles.value_color.to_string(),
              // The wind degrees character used is `⮕`, which is at 90°.
              rotate: (wind_deg() - 90.).to_string() + "deg",
              "⮕",
            }
          }
          rect {
            width: "100%",
            direction: "horizontal",
            main_align: "center",
            spacing: global_styles.h_gap.to_string(),
            label { font_family: "Noto Color Emoji", "☀️" }
            label { color: styles.value_color.to_string(), "{sunrise_text}" }
            label { font_family: "Noto Color Emoji", "🌙" }
            label { color: styles.value_color.to_string(), "{sunset_text}" }
          }
        }
      }
    }
  )
}
