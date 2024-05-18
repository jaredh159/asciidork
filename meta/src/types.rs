#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SafeMode {
  Unsafe,
  Safe,
  Server,
  #[default]
  Secure,
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
    let mut name = String::with_capacity(
      self.first_name.len()
        + self.last_name.len()
        + self.middle_name.as_ref().map_or(0, |s| s.len())
        + 2,
    );
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
