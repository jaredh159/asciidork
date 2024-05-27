use crate::internal::*;

pub fn attr(key: impl Into<String>, value: &AttrValue) -> Result<(), String> {
  let key: String = key.into();
  match key.as_str() {
    "attribute-missing" => constrain_to(&key, value, &["skip", "warn", "drop", "drop-line"])?,
    "attribute-undefined" => constrain_to(&key, value, &["drop", "drop-line"])?,
    _ => {}
  }
  Ok(())
}

fn constrain_to(key: &str, value: &AttrValue, options: &[&str]) -> Result<(), String> {
  match value {
    AttrValue::String(value) => {
      if options.contains(&value.as_str()) {
        Ok(())
      } else {
        Err(format!(
          "Invalid value for attr `{}`, expected one of `{}`",
          key,
          options.join("`, `")
        ))
      }
    }
    _ => Err(format!(
      "Invalid value for attr `{}`, expected one of `{}`",
      key,
      options.join("`, `")
    )),
  }
}
