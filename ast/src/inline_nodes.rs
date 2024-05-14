use std::ops::{Deref, DerefMut};

use crate::internal::*;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InlineNodes<'bmp>(BumpVec<'bmp, InlineNode<'bmp>>);

impl<'bmp> InlineNodes<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    Self(BumpVec::new_in(bump))
  }

  pub fn plain_text(&self) -> Vec<&str> {
    let mut text = Vec::new();
    self.iter().for_each(|node| match &node.content {
      Inline::AttributeReference(_) => {}
      Inline::Bold(nodes) => text.extend(nodes.plain_text()),
      Inline::CurlyQuote(_) => {}
      Inline::Discarded => {}
      Inline::Highlight(nodes) => text.extend(nodes.plain_text()),
      Inline::Macro(_) => {}
      Inline::Italic(nodes) => text.extend(nodes.plain_text()),
      Inline::InlinePassthrough(nodes) => text.extend(nodes.plain_text()),
      Inline::Newline => text.push(" "),
      Inline::LineBreak => {}
      Inline::LineComment(_) => {}
      Inline::CalloutNum(_) => {}
      Inline::LitMono(string) => text.push(string),
      Inline::Mono(nodes) => text.extend(nodes.plain_text()),
      Inline::MultiCharWhitespace(_) => text.push(" "),
      Inline::Quote(_, nodes) => text.extend(nodes.plain_text()),
      Inline::SpecialChar(_) => {}
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

  pub fn last_loc_end(&self) -> Option<usize> {
    self.last().map(|node| node.loc.end)
  }

  pub fn remove_trailing_newline(&mut self) {
    if matches!(self.last().map(|n| &n.content), Some(Inline::Newline)) {
      self.pop();
    }
  }

  pub fn discard_trailing_newline(&mut self) {
    if matches!(self.last().map(|n| &n.content), Some(Inline::Newline)) {
      let idx = self.len() - 1;
      self.0[idx].content = Inline::Discarded;
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

  pub fn into_vec(self) -> BumpVec<'bmp, InlineNode<'bmp>> {
    self.0
  }
}

impl<'bmp> Deref for InlineNodes<'bmp> {
  type Target = BumpVec<'bmp, InlineNode<'bmp>>;
  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<'bmp> DerefMut for InlineNodes<'bmp> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

impl<'bmp> From<BumpVec<'bmp, InlineNode<'bmp>>> for InlineNodes<'bmp> {
  fn from(vec: BumpVec<'bmp, InlineNode<'bmp>>) -> Self {
    Self(vec)
  }
}

impl Json for InlineNodes<'_> {
  fn to_json_in(&self, buf: &mut JsonBuf) {
    self.0.to_json_in(buf);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::{assert_eq, *};

  #[test]
  fn test_plain_text() {
    let heading: InlineNodes = nodes![
      node!(Inline::Bold(just!("Document", 1..8)), 0..9),
      node!(" "; 9..10),
      node!(Inline::Italic(just!("title", 12..18)), 11..19),
    ];
    assert_eq!(heading.plain_text(), vec!["Document", " ", "title"]);
  }
}
