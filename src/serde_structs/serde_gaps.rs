use freya::prelude::Gaps;

use std::ops::Deref;

use serde::{Deserialize, Deserializer};

#[derive(Clone, Debug, PartialEq)]
pub struct SerdeGaps(Gaps);

impl Deref for SerdeGaps {
  type Target = Gaps;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

/// Parse a string with space separated values.
/// - `all_sides`
/// - `top_and_bottom left_and_right`
/// - `top left_and_right bottom`
/// - `top right bottom left`
/// Parse a string with space separated values.
/// - `all_sides`
/// - `top_and_bottom left_and_right`
/// - `top left_and_right bottom`
/// - `top right bottom left`
fn parse_gaps(s: &str) -> Result<Gaps, Box<dyn std::error::Error>> {
  let s = s.trim();
  if s.is_empty() {
    return Ok(Gaps::default());
  }

  let parts: Vec<&str> = s.split_whitespace().collect();

  let parse_f32 = |t: &str| t.parse::<f32>();

  match parts.len() {
    1 => {
      let v = parse_f32(parts[0])?;
      Ok(Gaps::new_all(v))
    }
    2 => {
      let tb = parse_f32(parts[0])?;
      let lr = parse_f32(parts[1])?;
      Ok(Gaps::new(tb, lr, tb, lr))
    }
    3 => {
      let t = parse_f32(parts[0])?;
      let lr = parse_f32(parts[1])?;
      let b = parse_f32(parts[2])?;
      Ok(Gaps::new(t, lr, b, lr))
    }
    4 => {
      let t = parse_f32(parts[0])?;
      let r = parse_f32(parts[1])?;
      let b = parse_f32(parts[2])?;
      let l = parse_f32(parts[3])?;
      Ok(Gaps::new(t, r, b, l))
    }
    _ => Err("Invalid number of parameters".into()),
  }
}

impl<'de> Deserialize<'de> for SerdeGaps {
  fn deserialize<D>(deserializer: D) -> Result<SerdeGaps, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s: &str = Deserialize::deserialize(deserializer)?;
    match parse_gaps(s) {
      Ok(gaps) => Ok(SerdeGaps(gaps)),
      Err(err) => Err(serde::de::Error::custom(err.to_string())),
    }
  }
}
