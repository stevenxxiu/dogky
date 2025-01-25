use std::ops::Deref;

use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq)]
pub struct SerdeCommand(Vec<String>);

impl Deref for SerdeCommand {
  type Target = Vec<String>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'de> Deserialize<'de> for SerdeCommand {
  fn deserialize<D>(deserializer: D) -> Result<SerdeCommand, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    match shlex::split(s) {
      Some(parts) => Ok(SerdeCommand(parts)),
      None => Err(serde::de::Error::custom("shlex::split()")),
    }
  }
}
