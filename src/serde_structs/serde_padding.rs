use std::ops::Deref;

use iced::Padding;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub struct SerdePadding(Padding);

impl Deref for SerdePadding {
  type Target = Padding;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl From<SerdePadding> for Padding {
  fn from(item: SerdePadding) -> Self {
    item.0
  }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PaddingObj {
  F32(f32),
  VecF32(Vec<f32>),
}

impl<'de> Deserialize<'de> for SerdePadding {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let xs: PaddingObj = Deserialize::deserialize(deserializer)?;
    match xs {
      PaddingObj::F32(value) => Ok(SerdePadding(Padding::from(value))),
      PaddingObj::VecF32(value) => {
        if value.len() == 2 {
          Ok(SerdePadding(Padding::from([value[0], value[1]])))
        } else if value.len() == 4 {
          Ok(SerdePadding(Padding {
            top: value[0],
            right: value[1],
            bottom: value[2],
            left: value[3],
          }))
        } else {
          Err(serde::de::Error::custom("Invalid padding"))
        }
      }
    }
  }
}
