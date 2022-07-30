use std::error::Error;
use std::time::Duration;

use reqwest::blocking::ClientBuilder;
use serde_derive::{Deserialize, Serialize};

// Docs are at https://openweathermap.org/current#current_JSON. Internal parameters are excluded, as they might not
// exist.

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataCoord {
  lon: f64,
  lat: f64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataWeather {
  id: u64,
  main: String,
  description: String,
  icon: String,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataMain {
  temp: f64,
  feels_like: f64,
  temp_min: f64,
  temp_max: f64,
  pressure: f64,
  humidity: f64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataWind {
  speed: f64,
  deg: f64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataClouds {
  all: f64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
struct WeatherDataSys {
  country: String,
  sunrise: u64,
  sunset: u64,
}

#[allow(dead_code)]
#[derive(Clone, Serialize, Deserialize)]
pub struct WeatherData {
  coord: WeatherDataCoord,
  weather: Vec<WeatherDataWeather>,
  main: WeatherDataMain,
  visibility: f64,
  wind: WeatherDataWind,
  clouds: WeatherDataClouds,
  dt: u64,
  sys: WeatherDataSys,
  timezone: i32,
  id: u64,
  name: String,
}

static REQUEST_TIMEOUT: Duration = Duration::new(5, 0);

/// Use *Reqwest*'s blocking API, as *Reqwest* uses *Tokio* instead of *Rust* async.
pub fn get_weather(city_id: u64, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
  let request_url = format!(
    "https://api.openweathermap.org/data/2.5/weather?id={city_id}&units={units}&APPID={api_key}",
    city_id = city_id,
    units = "metric",
    api_key = api_key
  );
  let client = ClientBuilder::new().timeout(REQUEST_TIMEOUT).build()?;
  let response_text = client.get(&request_url).send()?.text()?;
  let res: WeatherData = serde_json::from_str(&response_text)?;
  Ok(res)
}
