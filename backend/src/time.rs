// replicates Ruby's Date.parse function, accepting arbitrary date string inputs.
pub fn format_date_str(input: &str, output_fmt: &str) -> Option<String> {
  let dt = dateparser::parse(input).ok()?;
  Some(dt.format(output_fmt).to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_date_to_iso() {
    assert_eq!(
      format_date_str("October 31, 2021", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
    assert_eq!(
      format_date_str("January 1, 2024", "%Y-%m-%d"),
      Some("2024-01-01".to_string())
    );
    assert_eq!(
      format_date_str("2021-10-31", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
  }

  #[test]
  fn test_parse_date_multiple_input_formats() {
    assert_eq!(
      format_date_str("October 31, 2021", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
    assert_eq!(
      format_date_str("2021-10-31", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
    assert_eq!(
      format_date_str("10/31/2021", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
    assert_eq!(
      format_date_str("Oct 31, 2021", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
  }

  #[test]
  fn test_parse_date_different_output_formats() {
    assert_eq!(
      format_date_str("October 31, 2021", "%Y-%m-%d"),
      Some("2021-10-31".to_string())
    );
    assert_eq!(
      format_date_str("October 31, 2021", "%m/%d/%Y"),
      Some("10/31/2021".to_string())
    );
    assert_eq!(
      format_date_str("October 31, 2021", "%B %d, %Y"),
      Some("October 31, 2021".to_string())
    );
    assert_eq!(
      format_date_str("October 31, 2021", "%b %d, %Y"),
      Some("Oct 31, 2021".to_string())
    );
  }

  #[test]
  fn test_parse_date_invalid() {
    assert_eq!(format_date_str("invalid date", "%Y-%m-%d"), None);
    assert_eq!(format_date_str("not a date", "%Y-%m-%d"), None);
  }

  #[test]
  fn test_parse_date_leap_year() {
    assert_eq!(
      format_date_str("February 29, 2024", "%Y-%m-%d"),
      Some("2024-02-29".to_string())
    );
    assert_eq!(format_date_str("February 29, 2023", "%Y-%m-%d"), None);
  }
}
