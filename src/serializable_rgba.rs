use std::fmt;
use std::ops::Deref;

use gtk4::gdk::RGBA;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

struct RGBAVisitor;

#[derive(Clone, Copy, Debug)]
pub struct SerializableRGBA(RGBA);

impl Deref for SerializableRGBA {
  type Target = RGBA;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl fmt::Display for SerializableRGBA {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Serialize for SerializableRGBA {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str((*self).to_str().as_str())
  }
}

impl<'de> Visitor<'de> for RGBAVisitor {
  type Value = SerializableRGBA;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a RGBA value")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(SerializableRGBA(RGBA::parse(value).unwrap()))
  }
}

impl<'de> Deserialize<'de> for SerializableRGBA {
  fn deserialize<D>(deserializer: D) -> Result<SerializableRGBA, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_string(RGBAVisitor)
  }
}
