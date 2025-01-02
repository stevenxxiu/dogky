use std::fs::File;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, FixedOffset};
use heck::ToTitleCase;
use iced::advanced::mouse;
use iced::alignment::{Horizontal, Vertical};
use iced::mouse::Interaction;
use iced::widget::{canvas, column, container, mouse_area, row, text, Text};
use iced::{alignment, Color, Element, Length, Point, Rectangle, Renderer, Subscription, Theme, Vector};
use iced::{time, Task};

use phf::phf_map;

use crate::api::{get_weather, WeatherData};
use crate::config::WeatherProps;
use crate::message::Message;
use crate::path::get_xdg_dirs;
use crate::styles::weather as styles;
use crate::ui_utils::space_row;

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

#[derive(Debug)]
struct WindArrow {
  offset: Vector,
  size: f32,
  angle: f32, // In degrees
  color: Color,
  cache: canvas::Cache,
}

impl WindArrow {
  fn new(offset: Vector, size: f32, angle: f32, color: Color) -> Self {
    Self {
      offset,
      size,
      angle,
      color,
      cache: canvas::Cache::new(),
    }
  }
}

impl<Message> canvas::Program<Message> for WindArrow {
  type State = ();

  fn draw(
    &self,
    _state: &(),
    renderer: &Renderer,
    _theme: &Theme,
    bounds: Rectangle,
    _cursor: mouse::Cursor,
  ) -> Vec<canvas::Geometry> {
    let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
      frame.translate(self.offset);

      // The data is offset by -180°. The wind degrees character used is `⮕`, which is at 90°.
      let angle = self.angle + 180. - 90.;
      frame.rotate(angle * std::f32::consts::PI / 180.0);

      frame.fill_text(canvas::Text {
        position: Point::new(0.0, 0.0),
        color: self.color,
        size: self.size.into(),
        content: String::from("⮕"),
        horizontal_alignment: alignment::Horizontal::Center,
        vertical_alignment: alignment::Vertical::Center,
        shaping: text::Shaping::Advanced,
        ..canvas::Text::default()
      });
    });

    vec![geometry]
  }
}

fn format_sun_timestamp(timestamp: u64, timezone: FixedOffset) -> String {
  let date_time = DateTime::from_timestamp(timestamp as i64, 0).unwrap() + timezone;
  date_time.format("%-l:%M %p").to_string()
}

pub struct WeatherWidget {
  config_props: WeatherProps,
  cache_path: PathBuf,
  data: Option<WeatherData>,
  error_str: Option<String>,
}

impl WeatherWidget {
  pub fn new(config_props: WeatherProps) -> Self {
    let mut res = Self {
      config_props,
      cache_path: get_xdg_dirs().place_cache_file("weather.json").unwrap(),
      data: None,
      error_str: None,
    };
    let _ = res.update(Message::WeatherWidgetTick);
    res
  }

  fn load_cache(&self) -> WeatherData {
    let data_file = File::open(&self.cache_path).unwrap();
    serde_json::from_reader(data_file).unwrap()
  }

  fn update_data(&mut self) {
    let props = &self.config_props;

    // No need to fetch data from server if cache time is close enough
    if let Ok(metadata) = std::fs::metadata(&self.cache_path) {
      let cache_time = metadata.modified().unwrap();
      let time_since_cache = SystemTime::now().duration_since(cache_time).unwrap();
      if time_since_cache < Duration::from_secs(props.update_interval) {
        self.data = Some(self.load_cache());
      }
    }

    // Update data from server
    match get_weather(props.openweather_city_id, &props.openweather_api_key) {
      Ok(weather_data) => {
        let data_file = File::create(&self.cache_path).unwrap();
        serde_json::to_writer(data_file, &weather_data).unwrap();
        self.data = Some(weather_data);
        self.error_str = None;
      }
      Err(error) => {
        self.error_str = Some(error.to_string());
      }
    }
  }

  pub fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::WeatherWidgetTick => self.update_data(),
      Message::WeatherWidgetClick => {
        // Open weather forecast link
        let props = &self.config_props;
        open::that(format!(
          "https://openweathermap.org/city/{0}#weather-widget",
          props.openweather_city_id
        ))
        .unwrap();
      }
      _ => {}
    }
    Task::none()
  }

  pub fn subscription(&self) -> Subscription<Message> {
    let props = &self.config_props;
    let timeout = if self.error_str.is_none() {
      props.update_interval
    } else {
      props.retry_timeout
    };
    time::every(Duration::from_secs(timeout)).map(|_instant| Message::WeatherWidgetTick)
  }

  pub fn view(&self) -> Element<Message> {
    macro_rules! space_row {
      ($row:expr) => {
        $row.spacing(styles::SPACING)
      };
    }

    let content = if let Some(error_str) = &self.error_str {
      column![space_row!(row![
        text("Weather: "),
        text(error_str).wrapping(text::Wrapping::Glyph),
      ])]
      .align_x(Horizontal::Center)
    } else {
      let data = self.data.as_ref().unwrap();

      let value_text = |s: String| -> Text { text(s).color(styles::TEXT_COLOR) };

      let cond_icon_key: String = data.weather[0].icon.chars().take(2).collect();
      let cond_icon = *ICON_MAP.get(cond_icon_key.as_str()).unwrap();
      let cond_icon_text: Text = text(cond_icon).font(styles::ICON_FONT).size(styles::COND_ICON_SIZE);
      let cond_icon_cont = container(cond_icon_text).padding(styles::COND_ICON_PADDING);

      let conditions = data.weather[0].description.to_title_case();
      let temperature = format!("{:.0}°C", data.main.temp);

      let humidity = format!("{}%", data.main.humidity);
      let wind_speed = format!("{:.1} m/s", data.wind.speed);

      let wind_arrow_offset = Vector::new(styles::WIND_ARROW_OFFSET, styles::WIND_ARROW_OFFSET);
      let wind_arrow = canvas(WindArrow::new(
        wind_arrow_offset,
        styles::WIND_ARROW_SIZE,
        data.wind.deg,
        styles::TEXT_COLOR,
      ))
      .width(styles::WIND_ARROW_CANVAS_SIZE)
      .height(styles::WIND_ARROW_CANVAS_SIZE);

      let timezone = FixedOffset::east_opt(data.timezone).unwrap();
      let sunrise_icon_text = text("☀️").font(styles::ICON_FONT);
      let sunrise_icon_cont = container(sunrise_icon_text).padding(styles::SUNRISE_ICON_PADDING);
      let sunrise_text = format_sun_timestamp(data.sys.sunrise, timezone);
      let sunset_icon_text = text("🌙").font(styles::ICON_FONT);
      let sunset_icon_cont = container(sunset_icon_text).padding(styles::SUNSET_ICON_PADDING);
      let sunset_text = format_sun_timestamp(data.sys.sunset, timezone);

      column![
        space_row![row![cond_icon_cont, text(conditions), value_text(temperature)].align_y(Vertical::Center)],
        space_row![row![
          text("Humidity"),
          value_text(humidity),
          text("Wind"),
          value_text(wind_speed),
          wind_arrow
        ]],
        space_row![row![
          sunrise_icon_cont,
          value_text(sunrise_text),
          sunset_icon_cont,
          value_text(sunset_text)
        ]],
      ]
      .align_x(Horizontal::Center)
    };
    let cur_container = container(content)
      .center_x(Length::Fill)
      .padding(styles::CONTAINER_PADDING);
    mouse_area(cur_container)
      .interaction(Interaction::Pointer)
      .on_press(Message::WeatherWidgetClick)
      .into()
  }
}
