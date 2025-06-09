use std::convert::TryInto;

use jiff::civil::Date;
use jiff::{tz::TimeZone, Timestamp, Zoned};

use crate::internal::*;

impl Parser<'_> {
  pub(crate) fn set_datetime_attrs(
    &mut self,
    now: u64,
    input_mtime: Option<u64>,
    reproducible: Option<u64>,
  ) {
    let doc_ts = reproducible.unwrap_or(input_mtime.unwrap_or(now));
    let now_ts = reproducible.unwrap_or(now);
    let docdate = date_from_attr_or(self.document.meta.str("docdate"), doc_ts);
    let doctime = time_from_attr_or(self.document.meta.str("doctime"), doc_ts);

    let docyear_s = docdate.year().to_string();
    let docdate_s = docdate.strftime("%Y-%m-%d").to_string();
    let doctime_s = doctime.strftime("%H:%M:%S%z").to_string();
    let docdatetime_s = format!("{} {}", docdate_s, doctime_s);
    self.insert_job_attr("docyear", docyear_s);
    self.insert_job_attr("docdate", docdate_s);
    self.insert_job_attr("doctime", doctime_s);
    self.insert_job_attr("docdatetime", docdatetime_s);

    let nowdatetime = to_zoned(now_ts);
    let nowyear = nowdatetime.year().to_string();
    let nowdate = nowdatetime.strftime("%Y-%m-%d").to_string();
    let nowtime = nowdatetime.strftime("%H:%M:%S%z").to_string();
    let nowdatetime = nowdatetime.strftime("%Y-%m-%d %H:%M:%S%z").to_string();
    self.insert_job_attr("localyear", nowyear);
    self.insert_job_attr("localdate", nowdate);
    self.insert_job_attr("localtime", nowtime);
    self.insert_job_attr("localdatetime", nowdatetime);
  }
}

fn to_zoned(seconds: u64) -> Zoned {
  let seconds: i64 = seconds.try_into().expect("invalid timestamp");
  let timestamp = Timestamp::from_second(seconds).expect("invalid timestamp");
  timestamp.to_zoned(TimeZone::UTC)
}

fn date_from_attr_or(attr: Option<&str>, epoch_seconds: u64) -> Zoned {
  attr
    .and_then(|attr_str| Date::strptime("%Y-%m-%d", attr_str).ok())
    .map(|date| date.to_zoned(TimeZone::UTC).expect("invalid date"))
    .unwrap_or_else(|| to_zoned(epoch_seconds))
}

fn time_from_attr_or(attr: Option<&str>, epoch_seconds: u64) -> Zoned {
  attr
    .and_then(|s| Zoned::strptime("%Y-%m-%d %H:%M:%S%z", format!("2000-01-01 {s}")).ok())
    .unwrap_or_else(|| to_zoned(epoch_seconds))
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_set_datetime_attrs_no_attr_overrides() {
    let mut parser = test_parser!("");
    parser.provide_timestamps(1734960586, Some(1734872889), None);
    let meta = parser.document.meta;
    assert_eq!(meta.str("docyear"), Some("2024"));
    assert_eq!(meta.str("docdate"), Some("2024-12-22"));
    assert_eq!(meta.str("doctime"), Some("13:08:09+0000"));
    assert_eq!(meta.str("docdatetime"), Some("2024-12-22 13:08:09+0000"));
    assert_eq!(meta.str("localyear"), Some("2024"));
    assert_eq!(meta.str("localdate"), Some("2024-12-23"));
    assert_eq!(meta.str("localtime"), Some("13:29:46+0000"));
    assert_eq!(meta.str("localdatetime"), Some("2024-12-23 13:29:46+0000"));
  }

  #[test]
  fn test_override_datetime_attrs() {
    let mut parser = test_parser!("");
    parser.insert_doc_attr("docdate", "2015-01-01").unwrap();
    parser.insert_doc_attr("doctime", "10:00:00-0700").unwrap();
    parser.provide_timestamps(1734960586, Some(1734872889), None);
    let meta = parser.document.meta;
    assert_eq!(meta.str("docyear"), Some("2015"));
    assert_eq!(meta.str("docdate"), Some("2015-01-01"));
    assert_eq!(meta.str("doctime"), Some("10:00:00-0700"));
    assert_eq!(meta.str("docdatetime"), Some("2015-01-01 10:00:00-0700"));
    assert_eq!(meta.str("localdatetime"), Some("2024-12-23 13:29:46+0000"));
  }

  #[test]
  fn test_override_just_time_attrs() {
    let mut parser = test_parser!("");
    parser.insert_doc_attr("doctime", "10:00:00-0700").unwrap();
    parser.provide_timestamps(1734960586, Some(1734872889), None);
    let meta = parser.document.meta;
    assert_eq!(meta.str("docyear"), Some("2024"));
    assert_eq!(meta.str("docdate"), Some("2024-12-22"));
    assert_eq!(meta.str("doctime"), Some("10:00:00-0700"));
    assert_eq!(meta.str("docdatetime"), Some("2024-12-22 10:00:00-0700"));
    assert_eq!(meta.str("localdatetime"), Some("2024-12-23 13:29:46+0000"));
  }

  #[test]
  fn test_reproducible_override_wins() {
    let mut parser = test_parser!("");
    parser.provide_timestamps(1734960586, Some(1734872889), Some(1262304000));
    let meta = parser.document.meta;
    assert_eq!(meta.str("docyear"), Some("2010"));
    assert_eq!(meta.str("doctime"), Some("00:00:00+0000"));
    assert_eq!(meta.str("docdatetime"), Some("2010-01-01 00:00:00+0000"));
  }
}
