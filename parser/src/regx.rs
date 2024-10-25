use lazy_static::lazy_static;
use regex::Regex;

// attrs
lazy_static! {
  pub static ref ATTR_DECL: Regex = Regex::new(r"^:([^\s:]+):\s*([^\s].*)?$").unwrap();
  pub static ref ATTR_VAL_REPLACE: Regex = Regex::new(r"\{([^\s}]+)\}").unwrap();
}

// directives
lazy_static! {
  pub static ref DIRECTIVE_IFDEF: Regex =
    Regex::new(r#"^ifn?def::([^\[]+[^\[\s])\[(.*)\]$"#).unwrap();
  pub static ref DIRECTIVE_ENDIF: Regex = Regex::new(r#"^endif::(\S*)\[\]$"#).unwrap();
  pub static ref DIRECTIVE_INCLUDE: Regex =
    Regex::new(r#"^include::([^\[]+[^\[\s])\[.*\]$"#).unwrap();
}

// line
lazy_static! {
  pub static ref REPEAT_STAR_LI_START: Regex = Regex::new(r#"^\s?(\*+)\s+.+"#).unwrap();
}
