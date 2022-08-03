use std::error::Error;
use std::time::Duration;

use reqwest::blocking::ClientBuilder;
use serde_derive::{Deserialize, Serialize};

// Docs are at https://openweathermap.org/current#current_JSON. Internal parameters are excluded, as they might not
// exist.

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataCoord {
  pub lon: f64,
  pub lat: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataWeather {
  pub id: u64,
  pub main: String,
  pub description: String,
  pub icon: String,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataMain {
  pub temp: f64,
  pub feels_like: f64,
  pub temp_min: f64,
  pub temp_max: f64,
  pub pressure: f64,
  pub humidity: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataWind {
  pub speed: f64,
  pub deg: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataClouds {
  pub all: f64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherDataSys {
  pub country: String,
  pub sunrise: u64,
  pub sunset: u64,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct WeatherData {
  pub coord: WeatherDataCoord,
  pub weather: Vec<WeatherDataWeather>,
  pub main: WeatherDataMain,
  pub visibility: f64,
  pub wind: WeatherDataWind,
  pub clouds: WeatherDataClouds,
  pub dt: u64,
  pub sys: WeatherDataSys,
  pub timezone: i32,
  pub id: u64,
  pub name: String,
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
