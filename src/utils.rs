use humansize::file_size_opts::{FileSizeOpts, FixedAt, Kilo};

pub const FILE_SIZE_OPTS: FileSizeOpts = FileSizeOpts {
  divider: Kilo::Binary,
  units: Kilo::Binary,
  decimal_places: 1,
  decimal_zeroes: 1,
  fixed_at: FixedAt::No,
  long_units: false,
  space: true,
  suffix: "",
  allow_negative: false,
};

pub fn div_rem(a: u64, b: u64) -> (u64, u64) {
  (a / b, a % b)
}

pub fn format_duration(total_seconds: u64) -> String {
  let (minutes, seconds) = div_rem(total_seconds, 60);
  if minutes == 0 {
    return format!("{}s", seconds);
  }
  let (hours, minutes) = div_rem(minutes, 60);
  if hours == 0 {
    return format!("{}m {:0>2}s", minutes, seconds);
  }
  let (days, hours) = div_rem(hours, 24);
  if days == 0 {
    return format!("{}h {:0>2}m {:0>2}s", hours, minutes, seconds);
  }
  format!("{}d {:0>2}h {:0>2}m {:0>2}s", days, hours, minutes, seconds)
}
