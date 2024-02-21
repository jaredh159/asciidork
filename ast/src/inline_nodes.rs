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
      Inline::Bold(nodes) => text.extend(nodes.plain_text()),
      Inline::Curly(_) => {}
      Inline::Discarded => {}
      Inline::Highlight(nodes) => text.extend(nodes.plain_text()),
      Inline::Macro(_) => {}
      Inline::Italic(nodes) => text.extend(nodes.plain_text()),
      Inline::InlinePassthrough(nodes) => text.extend(nodes.plain_text()),
      Inline::JoiningNewline => text.push(" "),
      Inline::LitMono(string) => text.push(string),
      Inline::Mono(nodes) => text.extend(nodes.plain_text()),
      Inline::MultiCharWhitespace(_) => text.push(" "),
      Inline::Quote(_, nodes) => text.extend(nodes.plain_text()),
      Inline::SpecialChar(_) => {}
      Inline::Superscript(nodes) => text.extend(nodes.plain_text()),
      Inline::Subscript(nodes) => text.extend(nodes.plain_text()),
      Inline::Text(s) => text.push(s.as_str()),
      Inline::TextSpan(_, nodes) => text.extend(nodes.plain_text()),
    });
    text
  }

  pub fn last_loc_end(&self) -> Option<usize> {
    self.last().map(|node| node.loc.end)
  }

  pub fn remove_trailing_newline(&mut self) {
    if matches!(
      self.last().map(|n| &n.content),
      Some(Inline::JoiningNewline)
    ) {
      self.pop();
    }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_button_menu_macro() {
    let b = &Bump::new();
    let nodes: InlineNodes = bvec![in b;
      InlineNode::new(
        Inline::Bold(bvec![in b;
          InlineNode::new(
            Inline::Text(BumpString::from_str_in("Document", b)),
            SourceLocation::new(1, 8),
          ),
        ].into()),
        SourceLocation::new(0, 9),
      ),
      InlineNode::new(
        Inline::Text(BumpString::from_str_in(" ", b)),
        SourceLocation::new(9, 10),
      ),
      InlineNode::new(
        Inline::Italic(bvec![in b;
          InlineNode::new(
            Inline::Text(BumpString::from_str_in("title", b)),
            SourceLocation::new(12, 18),
          ),
        ].into()),
        SourceLocation::new(11, 19),
      ),
    ]
    .into();
    assert_eq!(nodes.plain_text(), vec!["Document", " ", "title"]);
  }
}
