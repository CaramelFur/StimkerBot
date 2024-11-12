use std::time::{SystemTime, UNIX_EPOCH};
use time_humanize::HumanTime;

pub fn get_unix() -> i64 {
  let now = SystemTime::now();
  let unix = now.duration_since(UNIX_EPOCH).unwrap();
  unix.as_millis() as i64
}

// Calculate humantime from now to unix timestamp in milliseconds
// e.g. "5 hours ago"
pub fn unix_to_humantime(unix: i64) -> String {
  if unix == 0 {
    return "never".to_string();
  }

  let humantime = HumanTime::from_duration_since_timestamp((unix / 1000).try_into().unwrap());
  humantime.to_string()
}
