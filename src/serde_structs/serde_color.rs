use std::ops::Deref;

use freya_core::parsing::Parse;
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
    match Color::parse(s) {
      Ok(color) => Ok(SerdeColor(color)),
      Err(_) => Err(serde::de::Error::custom("parse_hex_skia_color()")),
    }
  }
}
