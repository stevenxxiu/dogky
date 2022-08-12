use regex::Regex;
use std::fmt;
use std::ops::Deref;

use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

struct RegexVisitor;

#[derive(Clone, Debug)]
pub struct SerializableRegex(Regex);

impl Deref for SerializableRegex {
  type Target = Regex;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl fmt::Display for SerializableRegex {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Serialize for SerializableRegex {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str((*self).to_string().as_str())
  }
}

impl<'de> Visitor<'de> for RegexVisitor {
  type Value = SerializableRegex;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a regex value")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(SerializableRegex(Regex::new(value).unwrap()))
  }
}

impl<'de> Deserialize<'de> for SerializableRegex {
  fn deserialize<D>(deserializer: D) -> Result<SerializableRegex, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_string(RegexVisitor)
  }
}
