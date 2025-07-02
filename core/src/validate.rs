use crate::internal::*;

pub fn attr<A: RemoveAttr>(
  attrs: &mut A,
  key: impl Into<String>,
  value: &AttrValue,
) -> Result<(), String> {
  let key: String = key.into();
  match key.as_str() {
    "attribute-missing" => one_of(&["skip", "warn", "drop", "drop-line"], &key, value)?,
    "attribute-undefined" => one_of(&["drop", "drop-line"], &key, value)?,
    "showtitle" | "notitle" => bool(&key, value)?,
    _ => {}
  }
  if &key == "showtitle" {
    attrs.remove("notitle");
  } else if &key == "notitle" {
    attrs.remove("showtitle");
  }
  Ok(())
}

fn one_of(options: &[&str], key: &str, value: &AttrValue) -> Result<(), String> {
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

fn bool(key: impl Into<String>, value: &AttrValue) -> Result<(), String> {
  let key: String = key.into();
  match value {
    AttrValue::Bool(_) => Ok(()),
    _ => Err(format!(
      "Invalid value for attr `{key}`, expected empty string"
    )),
  }
}
