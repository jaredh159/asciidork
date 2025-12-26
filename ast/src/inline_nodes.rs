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
      Inline::Span(_, _, nodes) => text.extend(nodes.plain_text()),
      Inline::CurlyQuote(RightDouble) => text.push("”"),
      Inline::CurlyQuote(LeftDouble) => text.push("“"),
      Inline::CurlyQuote(LeftSingle) => text.push("‘"),
      Inline::CurlyQuote(RightSingle) => text.push("’"),
      Inline::CurlyQuote(LegacyImplicitApostrophe) => text.push("'"),
      Inline::Discarded => {}
      Inline::Macro(_) => {}
      Inline::InlinePassthru(nodes) => text.extend(nodes.plain_text()),
      Inline::Newline => text.push(" "),
      Inline::InlineAnchor(_) => {}
      Inline::IndexTerm(IndexTerm {
        term_type: IndexTermType::Visible { term },
        ..
      }) => text.extend(term.plain_text()),
      Inline::IndexTerm(..) => {}
      Inline::BiblioAnchor(..) => {}
      Inline::LineBreak => {}
      Inline::LineComment(_) => {}
      Inline::CalloutNum(_) => {}
      Inline::MultiCharWhitespace(_) => text.push(" "),
      Inline::Quote(_, nodes) => text.extend(nodes.plain_text()),
      Inline::SpacedDashes(2, _) => text.push(" — "),
      Inline::SpacedDashes(_, _) => text.push(" --- "),
      Inline::SpecialChar(SpecialCharKind::Ampersand) => text.push("&"),
      Inline::SpecialChar(SpecialCharKind::LessThan) => text.push("<"),
      Inline::SpecialChar(SpecialCharKind::GreaterThan) => text.push(">"),
      Inline::Symbol(SymbolKind::Copyright) => text.push("(C)"),
      Inline::Symbol(SymbolKind::Trademark) => text.push("(TM)"),
      Inline::Symbol(SymbolKind::Registered) => text.push("(R)"),
      Inline::Symbol(SymbolKind::EmDash) => text.push("—"),
      Inline::Symbol(SymbolKind::TripleDash) => text.push("---"),
      Inline::Symbol(SymbolKind::Ellipsis) => text.push("..."),
      Inline::Symbol(SymbolKind::SingleRightArrow) => text.push("->"),
      Inline::Symbol(SymbolKind::DoubleRightArrow) => text.push("=>"),
      Inline::Symbol(SymbolKind::SingleLeftArrow) => text.push("<-"),
      Inline::Symbol(SymbolKind::DoubleLeftArrow) => text.push("<="),
      Inline::Text(s) => text.push(s.as_str()),
      Inline::CalloutTuck(_) => {}
    });
    text
  }

  pub fn loc(&self) -> Option<MultiSourceLocation> {
    self
      .loc_span()
      .map(|(start, end)| MultiSourceLocation::spanning(start, end))
  }

  pub fn loc_span(&self) -> Option<(SourceLocation, SourceLocation)> {
    self.first_loc().zip(self.last_loc())
  }

  pub fn first_loc(&self) -> Option<SourceLocation> {
    self.first().map(|node| node.loc)
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

  pub fn trim_trailing_whitespace(&mut self) {
    let Some(last) = self.last_mut() else {
      return;
    };
    match &mut last.content {
      Inline::Text(s) => {
        while s.ends_with(char::is_whitespace) {
          s.pop();
          last.loc.end -= 1;
        }
      }
      Inline::MultiCharWhitespace(_) => {
        self.pop();
      }
      _ => {}
    }
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
      node!(
        Inline::Span(SpanKind::Bold, None, just!("Document", 1..8)),
        0..9
      ),
      node!(" "; 9..10),
      node!(
        Inline::Span(SpanKind::Italic, None, just!("title", 12..18)),
        11..19
      ),
    ];
    expect_eq!(heading.plain_text(), vec!["Document", " ", "title"]);
  }
}
