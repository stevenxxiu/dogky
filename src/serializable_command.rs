use std::fmt;
use std::ops::Deref;

use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

struct CommandVisitor;

#[derive(Clone, Debug)]
pub struct SerializableCommand(Vec<String>);

impl Deref for SerializableCommand {
  type Target = Vec<String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl fmt::Display for SerializableCommand {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl Serialize for SerializableCommand {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&shlex::join(self.0.iter().map(|s| s.as_str())).to_string().as_str())
  }
}

impl<'de> Visitor<'de> for CommandVisitor {
  type Value = SerializableCommand;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a shell command")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Ok(SerializableCommand(shlex::split(value).unwrap()))
  }
}

impl<'de> Deserialize<'de> for SerializableCommand {
  fn deserialize<D>(deserializer: D) -> Result<SerializableCommand, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_string(CommandVisitor)
  }
}
