use std::ops::Deref;

use csscolorparser::Color as CssColor;
use freya_engine::prelude::Color;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq)]
pub struct SerdeColor(Color);

impl Deref for SerdeColor {
  type Target = Color;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for SerdeColor {
  fn deserialize<D>(deserializer: D) -> Result<SerdeColor, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s.parse::<CssColor>() {
      Ok(color) => Ok(SerdeColor(Color::from_argb(
        (color.a * 255.) as u8,
        (color.r * 255.) as u8,
        (color.g * 255.) as u8,
        (color.b * 255.) as u8,
      ))),
      Err(_) => Err(serde::de::Error::custom("Invalid color string")),
    }
  }
}
