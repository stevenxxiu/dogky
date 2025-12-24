use std::error::Error;
use std::time::Duration;

use serde_derive::{Deserialize, Serialize};

// Docs are at https://openweathermap.org/current#current_JSON. Internal parameters are excluded, as they might not
// exist.

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataCoord {
  pub lon: f32,
  pub lat: f32,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataWeather {
  pub id: u64,
  pub main: String,
  pub description: String,
  pub icon: String,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataMain {
  pub temp: f32,
  pub feels_like: f32,
  pub temp_min: f32,
  pub temp_max: f32,
  pub pressure: f32,
  pub humidity: f32,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataWind {
  pub speed: f32,
  pub deg: f32,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataClouds {
  pub all: f32,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherDataSys {
  pub country: String,
  pub sunrise: u64,
  pub sunset: u64,
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct WeatherData {
  pub coord: WeatherDataCoord,
  pub weather: Vec<WeatherDataWeather>,
  pub main: WeatherDataMain,
  pub visibility: f32,
  pub wind: WeatherDataWind,
  pub clouds: WeatherDataClouds,
  pub dt: u64,
  pub sys: WeatherDataSys,
  pub timezone: i32,
  pub id: u64,
  pub name: String,
}

static REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn get_weather(city_id: u64, api_key: &str) -> Result<WeatherData, Box<dyn Error>> {
  let request_url = format!(
    "https://api.openweathermap.org/data/2.5/weather?id={city_id}&units={units}&APPID={api_key}",
    city_id = city_id,
    units = "metric",
    api_key = api_key
  );
  let client = reqwest::Client::builder().timeout(REQUEST_TIMEOUT).build()?;
  let response = client.get(&request_url).send().await?;
  let res: WeatherData = serde_json::from_str(&response.text().await?)?;
  Ok(res)
}
