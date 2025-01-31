use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::format_size::format_size;

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

pub const MEMORY_DECIMAL_PLACES: usize = 1usize;

pub fn format_used(used: u64, total: u64) -> String {
  format!(
    "{: >10}/{: >10} = {: >3.0}%",
    format_size(used, MEMORY_DECIMAL_PLACES),
    format_size(total, MEMORY_DECIMAL_PLACES),
    (used as f32) / (total as f32) * 100.0
  )
}

pub fn join_str_iter<'a, I>(str_iter: I, seperator: &str) -> String
where
  I: Iterator<Item = String>,
{
  str_iter.fold(String::new(), |res, cur| {
    if res.is_empty() {
      return cur.to_string();
    }
    res + seperator + &cur.to_string()
  })
}

pub fn substitute_env_vars(command: &str) -> String {
  lazy_static! {
    static ref RE_VAR: Regex = Regex::new(r"\$([a-zA-Z_]+[a-zA-Z\d_]*)|\$\{([a-zA-Z_]+[a-zA-Z\d_]*)}").unwrap();
  }
  RE_VAR
    .replace_all(command, |caps: &Captures| {
      let var_name = caps.get(1).or(caps.get(2)).unwrap().as_str();
      std::env::var(var_name).unwrap_or(caps[0].to_string())
    })
    .into_owned()
}
