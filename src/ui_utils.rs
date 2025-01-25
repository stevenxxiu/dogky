use freya::prelude::Gaps;
use skia_safe::Color;

pub fn parse_padding(value: &str) -> Result<Gaps, &str> {
  let mut paddings = Gaps::default();

  if value == "none" {
    return Ok(paddings);
  }

  let mut values = value.split_ascii_whitespace();
  let parse_error = "Parse error";

  match values.clone().count() {
    // Same in each directions
    1 => {
      paddings.fill_all(
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
      );
    }
    // By vertical and horizontal
    2 => {
      // Vertical
      paddings.fill_vertical(
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
      );

      // Horizontal
      paddings.fill_horizontal(
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
      )
    }
    // Individual vertical but same horizontal
    3 => {
      let top = values
        .next()
        .ok_or(parse_error)?
        .parse::<f32>()
        .map_err(|_| parse_error)?;
      let left_and_right = values
        .next()
        .ok_or(parse_error)?
        .parse::<f32>()
        .map_err(|_| parse_error)?;
      let bottom = values
        .next()
        .ok_or(parse_error)?
        .parse::<f32>()
        .map_err(|_| parse_error)?;
      paddings = Gaps::new(top, left_and_right, bottom, left_and_right);
    }
    // Each directions
    4 => {
      paddings = Gaps::new(
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
        values
          .next()
          .ok_or(parse_error)?
          .parse::<f32>()
          .map_err(|_| parse_error)?,
      );
    }
    _ => {}
  }

  Ok(paddings)
}

pub fn parse_hex_skia_color(color: &str) -> Result<Color, &str> {
  if color.len() == 7 {
    let r = u8::from_str_radix(&color[1..3], 16).map_err(|_| "Parse Error")?;
    let g = u8::from_str_radix(&color[3..5], 16).map_err(|_| "Parse Error")?;
    let b = u8::from_str_radix(&color[5..7], 16).map_err(|_| "Parse Error")?;
    Ok(Color::from_rgb(r, g, b))
  } else {
    Err("Parse error")
  }
}
