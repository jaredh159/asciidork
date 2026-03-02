use crate::short::block::*;

#[derive(Copy, Debug, PartialEq, Eq, Clone)]
pub enum AdmonitionKind {
  Tip,
  Caution,
  Important,
  Note,
  Warning,
}

impl AdmonitionKind {
  pub const fn lowercase_str(&self) -> &'static str {
    match self {
      AdmonitionKind::Tip => "tip",
      AdmonitionKind::Caution => "caution",
      AdmonitionKind::Important => "important",
      AdmonitionKind::Note => "note",
      AdmonitionKind::Warning => "warning",
    }
  }

  pub const fn str(&self) -> &'static str {
    match self {
      AdmonitionKind::Tip => "Tip",
      AdmonitionKind::Caution => "Caution",
      AdmonitionKind::Important => "Important",
      AdmonitionKind::Note => "Note",
      AdmonitionKind::Warning => "Warning",
    }
  }

  pub const fn caption_name(&self) -> &'static str {
    match self {
      AdmonitionKind::Tip => "tip-caption",
      AdmonitionKind::Caution => "caution-caption",
      AdmonitionKind::Important => "important-caption",
      AdmonitionKind::Note => "note-caption",
      AdmonitionKind::Warning => "warning-caption",
    }
  }
}

impl TryFrom<Context> for AdmonitionKind {
  type Error = &'static str;
  fn try_from(context: Context) -> Result<Self, Self::Error> {
    match context {
      Context::AdmonitionTip => Ok(AdmonitionKind::Tip),
      Context::AdmonitionCaution => Ok(AdmonitionKind::Caution),
      Context::AdmonitionImportant => Ok(AdmonitionKind::Important),
      Context::AdmonitionNote => Ok(AdmonitionKind::Note),
      Context::AdmonitionWarning => Ok(AdmonitionKind::Warning),
      _ => Err("not an admonition"),
    }
  }
}
