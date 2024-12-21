use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum SafeMode {
  Unsafe,
  Safe,
  Server,
  #[default]
  Secure,
}

impl FromStr for SafeMode {
  type Err = &'static str;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "unsafe" => Ok(SafeMode::Unsafe),
      "safe" => Ok(SafeMode::Safe),
      "server" => Ok(SafeMode::Server),
      "secure" => Ok(SafeMode::Secure),
      _ => Err("Invalid safe mode: expected `unsafe`, `safe`, `server`, or `secure`"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IconMode {
  #[default]
  Text,
  Image,
  Font,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Author {
  pub first_name: String,
  pub middle_name: Option<String>,
  pub last_name: String,
  pub email: Option<String>,
}

impl Author {
  pub fn fullname(&self) -> String {
    let mut name = String::with_capacity(64);
    name.push_str(&self.first_name);
    if let Some(middle_name) = &self.middle_name {
      name.push(' ');
      name.push_str(middle_name);
    }
    name.push(' ');
    name.push_str(&self.last_name);
    name
  }

  pub fn initials(&self) -> String {
    let mut initials = String::with_capacity(3);
    initials.push(self.first_name.chars().next().unwrap());
    if let Some(middle_name) = &self.middle_name {
      initials.push(middle_name.chars().next().unwrap());
    }
    initials.push(self.last_name.chars().next().unwrap());
    initials
  }
}
