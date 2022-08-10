const SIZE_UNITS: [(&str, i32); 9] = [
  ("B", 0),
  ("KiB", 10),
  ("MiB", 20),
  ("GiB", 30),
  ("TiB", 40),
  ("PiB", 50),
  ("EiB", 60),
  ("ZiB", 70),
  ("YiB", 80),
];

const POW_INC: u8 = 10u8;

pub fn format_size(size: u64, decimal_places: usize) -> String {
  let power = (size as f32).log2() / POW_INC as f32;
  let power_index = (power as usize).max(0).min(SIZE_UNITS.len() - 1);
  if power_index == 0 {
    return format!("{} B", size);
  }
  let (unit, base) = SIZE_UNITS[power_index];
  let significand = (size as f32) / 2f32.powi(base);
  format!(
    "{:.decimal_places$} {}",
    significand,
    unit,
    decimal_places = decimal_places
  )
}

pub fn format_speed(speed: f32, decimal_places: usize) -> String {
  let power = speed.log2() / POW_INC as f32;
  let power_index = (power as usize).max(0).min(SIZE_UNITS.len() - 1);
  let (unit, base) = SIZE_UNITS[power_index];
  let significand = speed / 2f32.powi(base);
  format!(
    "{:.decimal_places$} {}/s",
    significand,
    unit,
    decimal_places = decimal_places
  )
}
