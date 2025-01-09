use std::ops::Deref;

use regex::Regex;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub struct SerdeRegex(Regex);

impl Deref for SerdeRegex {
  type Target = Regex;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for SerdeRegex {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    Regex::new(s).map(SerdeRegex).map_err(serde::de::Error::custom)
  }
}
