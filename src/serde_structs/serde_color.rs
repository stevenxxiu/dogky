use std::ops::Deref;

use iced::Color;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub struct SerdeColor(Color);

impl Deref for SerdeColor {
  type Target = Color;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<SerdeColor> for Color {
  fn from(item: SerdeColor) -> Self {
    item.0
  }
}

impl<'de> Deserialize<'de> for SerdeColor {
  fn deserialize<D>(deserializer: D) -> Result<SerdeColor, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    csscolorparser::parse(s)
      .map(|color| SerdeColor(Color::from_rgba(color.r, color.g, color.b, color.a)))
      .map_err(serde::de::Error::custom)
  }
}
