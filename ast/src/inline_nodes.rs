use std::ops::{Deref, DerefMut};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNodes<'arena>(BumpVec<'arena, InlineNode<'arena>>);

impl<'arena> InlineNodes<'arena> {
  pub fn new(bump: &'arena Bump) -> Self {
    Self(BumpVec::new_in(bump))
  }

  pub fn plain_text(&self) -> Vec<&str> {
    let mut text = Vec::new();
    self.iter().for_each(|node| match &node.content {
      Inline::Bold(nodes) => text.extend(nodes.plain_text()),
      Inline::CurlyQuote(RightDouble) => text.push("”"),
      Inline::CurlyQuote(LeftDouble) => text.push("“"),
      Inline::CurlyQuote(LeftSingle) => text.push("‘"),
      Inline::CurlyQuote(RightSingle) => text.push("’"),
      Inline::CurlyQuote(LegacyImplicitApostrophe) => text.push("'"),
      Inline::Discarded => {}
      Inline::Highlight(nodes) => text.extend(nodes.plain_text()),
      Inline::Macro(_) => {}
      Inline::Italic(nodes) => text.extend(nodes.plain_text()),
      Inline::InlinePassthru(nodes) => text.extend(nodes.plain_text()),
      Inline::Newline => text.push(" "),
      Inline::InlineAnchor(_) => {}
      Inline::BiblioAnchor(..) => {}
      Inline::LineBreak => {}
      Inline::LineComment(_) => {}
      Inline::CalloutNum(_) => {}
      Inline::LitMono(string) => text.push(string),
      Inline::Mono(nodes) => text.extend(nodes.plain_text()),
      Inline::MultiCharWhitespace(_) => text.push(" "),
      Inline::Quote(_, nodes) => text.extend(nodes.plain_text()),
      Inline::SpecialChar(SpecialCharKind::Ampersand) => text.push("&"),
      Inline::SpecialChar(SpecialCharKind::LessThan) => text.push("<"),
      Inline::SpecialChar(SpecialCharKind::GreaterThan) => text.push(">"),
      Inline::Symbol(SymbolKind::Copyright) => text.push("(C)"),
      Inline::Symbol(SymbolKind::Trademark) => text.push("(TM)"),
      Inline::Symbol(SymbolKind::Registered) => text.push("(R)"),
      Inline::Symbol(SymbolKind::EmDash) => text.push("—"),
      Inline::Symbol(SymbolKind::SpacedEmDash(_)) => text.push(" — "),
      Inline::Symbol(SymbolKind::Ellipsis) => text.push("..."),
      Inline::Symbol(SymbolKind::SingleRightArrow) => text.push("->"),
      Inline::Symbol(SymbolKind::DoubleRightArrow) => text.push("=>"),
      Inline::Symbol(SymbolKind::SingleLeftArrow) => text.push("<-"),
      Inline::Symbol(SymbolKind::DoubleLeftArrow) => text.push("<="),
      Inline::Superscript(nodes) => text.extend(nodes.plain_text()),
      Inline::Subscript(nodes) => text.extend(nodes.plain_text()),
      Inline::Text(s) => text.push(s.as_str()),
      Inline::TextSpan(_, nodes) => text.extend(nodes.plain_text()),
      Inline::CalloutTuck(_) => {}
    });
    text
  }

  pub fn last_loc(&self) -> Option<SourceLocation> {
    self.last().map(|node| node.loc)
  }

  pub fn last_loc_end(&self) -> Option<u32> {
    self.last().map(|node| node.loc.end)
  }

  pub fn remove_trailing_newline(&mut self) -> bool {
    if matches!(self.last().map(|n| &n.content), Some(Inline::Newline)) {
      self.pop();
      true
    } else {
      false
    }
  }

  pub fn remove_trailing_line_comment(&mut self) -> bool {
    if matches!(
      self.last().map(|n| &n.content),
      Some(Inline::LineComment(_))
    ) {
      self.pop();
      true
    } else {
      false
    }
  }

  pub fn discard_trailing_newline(&mut self) -> bool {
    if matches!(self.last().map(|n| &n.content), Some(Inline::Newline)) {
      let idx = self.len() - 1;
      self.0[idx].content = Inline::Discarded;
      true
    } else {
      false
    }
  }

  pub fn single_text(&self) -> Option<&str> {
    if self.len() == 1 {
      match &self[0].content {
        Inline::Text(s) => Some(s.as_str()),
        _ => None,
      }
    } else {
      None
    }
  }

  pub fn into_vec(self) -> BumpVec<'arena, InlineNode<'arena>> {
    self.0
  }

  pub fn nth(&self, n: usize) -> Option<&InlineNode<'arena>> {
    self.get(n)
  }

  pub fn last_is(&self, kind: &Inline) -> bool {
    self.last().is_some_and(|node| &node.content == kind)
  }
}

impl<'arena> Deref for InlineNodes<'arena> {
  type Target = BumpVec<'arena, InlineNode<'arena>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for InlineNodes<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<'arena> From<BumpVec<'arena, InlineNode<'arena>>> for InlineNodes<'arena> {
  fn from(vec: BumpVec<'arena, InlineNode<'arena>>) -> Self {
    Self(vec)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_plain_text() {
    let heading: InlineNodes = nodes![
      node!(Inline::Bold(just!("Document", 1..8)), 0..9),
      node!(" "; 9..10),
      node!(Inline::Italic(just!("title", 12..18)), 11..19),
    ];
    expect_eq!(heading.plain_text(), vec!["Document", " ", "title"]);
  }
}
