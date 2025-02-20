use lazy_static::lazy_static;
use regex::Regex;

// attrs
lazy_static! {
  pub static ref ATTR_DECL: Regex = Regex::new(r"^:([^\s:]+):\s*([^\s].*)?$").unwrap();
  pub static ref ATTR_VAL_REPLACE: Regex = Regex::new(r"\{([^\s}]+)\}").unwrap();
}

// email
lazy_static! {
  pub static ref EMAIL_RE: Regex = Regex::new(
    r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
  )
  .unwrap();
}

// directives
lazy_static! {
  pub static ref DIRECTIVE_INCLUDE: Regex =
    Regex::new(r#"^include::([^\[]+[^\[\s])\[.*\]$"#).unwrap();
  pub static ref DIRECTIVE_IFDEF: Regex =
    Regex::new(r#"^ifn?def::([^\[]+[^\[\s])\[(.*)\]$"#).unwrap();
  pub static ref DIRECTIVE_ENDIF: Regex = Regex::new(r#"^endif::(\S*)\[\]$"#).unwrap();
}

// ifeval directives
lazy_static! {
  pub static ref DIRECTIVE_IFEVAL: Regex =
    Regex::new(r#"^ifeval::(.*?)\[(.+?) *([=!><]=|[><]) *(.+)\]$"#).unwrap();
  pub static ref DIRECTIVE_INVALID_IFEVAL: Regex = Regex::new(r#"^ifeval::(\[(.*)\])$"#).unwrap();
}

// line
lazy_static! {
  pub static ref REPEAT_STAR_LI_START: Regex = Regex::new(r#"^\s?(\*+)\s+.+"#).unwrap();
}

// inlines
lazy_static! {
  pub static ref KBD_MACRO_KEYS: Regex = Regex::new(r"(?:\s*([^\s,+]+|[,+])\s*)").unwrap();
}
