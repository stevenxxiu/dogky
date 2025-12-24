use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use async_io::Timer;
use chrono::{DateTime, FixedOffset};
use freya::prelude::*;
use futures_lite::stream::StreamExt;
use heck::ToTitleCase;

use phf::phf_map;

use crate::api::{get_weather, WeatherData};
use crate::config::WeatherConfig;
use crate::freya_utils::{center_cont_factory, color_label, cursor_area, emoji_label, value_label_factory};
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

async fn update_data(config: &WeatherConfig, cache_path: &PathBuf) -> Result<WeatherData, String> {
  // No need to fetch data from server if cache time is close enough
  if let Ok(metadata) = std::fs::metadata(cache_path) {
    let cache_time = metadata.modified().unwrap();
    let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
    if time_since_cache < Duration::from_secs(config.update_interval) {
      return Ok(load_cache(cache_path.to_path_buf()));
    }
  }

  // Update data from server
  match get_weather(config.openweather_city_id, &config.openweather_api_key).await {
    Ok(weather_data) => {
      let data_file = File::create(cache_path).unwrap();
      serde_json::to_writer(data_file, &weather_data).unwrap();
      Ok(weather_data)
    }
    Err(error) => Err(error.to_string()),
  }
}

pub fn weather_component() -> CursorArea {
  let config = use_consume::<WeatherConfig>();
  let styles = use_consume::<WeatherStyles>();
  let global_styles = use_consume::<GlobalStyles>();
  let cache_path = get_xdg_dirs().place_cache_file("weather.json").unwrap();

  let mut has_err = use_state(|| false);
  let mut error_str = use_state(|| "".into());

  let city_id = config.openweather_city_id;
  let mut data = use_state(WeatherData::default);
  let mut cond_icon = use_state(|| "");

  use_hook(|| {
    spawn(async move {
      loop {
        let timeout = match update_data(&config, &cache_path).await {
          Ok(data_) => {
            data.set(data_);
            has_err.set(false);

            let cond_icon_key = data.read().weather[0].icon.chars().take(2).collect::<String>();
            cond_icon.set(ICON_MAP.get(cond_icon_key.as_str()).unwrap());

            config.update_interval
          }
          Err(error_str_) => {
            has_err.set(true);
            error_str.set(error_str_);
            config.retry_timeout
          }
        };
        Timer::interval(Duration::from_secs(timeout)).next().await;
      }
    });
  });

  let timezone = FixedOffset::east_opt(data.read().timezone).unwrap();

  let center_cont = center_cont_factory(global_styles.h_gap);
  let value_label = value_label_factory((*styles.value_color).into());

  cursor_area(CursorIcon::Pointer).child(
    rect()
      .width(Size::percent(100.))
      .padding(*styles.container_padding)
      .on_pointer_press(move |_| {
        // Open weather forecast link
        open::that(format!("https://openweathermap.org/city/{0}#weather-widget", city_id)).unwrap();
      })
      .children(if has_err() || data.read().weather.is_empty() {
        vec![center_cont(vec![
          label().text("Weather:").into(),
          label().text(error_str.read().to_string()).into(),
        ])
        .into()]
      } else {
        vec![
          center_cont(vec![
            emoji_label(cond_icon.read().to_string())
              .font_size(styles.cond_icon_size)
              .into(),
            label().text(data.read().weather[0].description.to_title_case()).into(),
            value_label(format!("{:.0}°C", data.read().main.temp)).into(),
          ])
          .cross_align(Alignment::Center)
          .into(),
          center_cont(vec![
            label().text("Humidity").into(),
            value_label(format!("{}%", data.read().main.humidity)).into(),
            label().text("Wind").into(),
            value_label(format!("{:.1} m/s", data.read().wind.speed)).into(),
            rect()
              .margin(*styles.wind_arrow_margin)
              .child(color_label(*styles.value_color, "⮕"))
              // The wind degrees character used is `⮕`, which is at 90°.
              .rotate(data.read().wind.deg - 90.)
              .into(),
          ])
          .into(),
          center_cont(vec![
            emoji_label("☀️").into(),
            value_label(format_sun_timestamp(data.read().sys.sunrise, timezone)).into(),
            emoji_label("🌙").into(),
            value_label(format_sun_timestamp(data.read().sys.sunset, timezone)).into(),
          ])
          .into(),
        ]
      }),
  )
}
