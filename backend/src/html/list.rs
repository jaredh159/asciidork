pub const fn type_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "1",
    2 => "a",
    3 => "i",
    4 => "A",
    _ => "I",
  }
}

pub fn type_from_class(class: &str) -> Option<&'static str> {
  match class {
    "arabic" => Some("1"),
    "loweralpha" => Some("a"),
    "lowerroman" => Some("i"),
    "upperalpha" => Some("A"),
    "upperroman" => Some("I"),
    _ => None,
  }
}

pub const fn class_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "arabic",
    2 => "loweralpha",
    3 => "lowerroman",
    4 => "upperalpha",
    _ => "upperroman",
  }
}
