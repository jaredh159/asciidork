use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'arena> {
  pub positional: BumpVec<'arena, Option<InlineNodes<'arena>>>,
  pub named: Named<'arena>,
  pub id: Option<SourceString<'arena>>,
  pub roles: BumpVec<'arena, SourceString<'arena>>,
  pub options: BumpVec<'arena, SourceString<'arena>>,
  pub loc: SourceLocation,
}

pub trait AttrData {
  fn is_empty(&self) -> bool;
  fn str_positional_at(&self, index: usize) -> Option<&str>;
  fn has_option(&self, option: &str) -> bool;
  fn has_str_positional(&self, positional: &str) -> bool;
  fn is_source(&self) -> bool;
  fn source_language(&self) -> Option<&str>;
  fn has_role(&self, role: &str) -> bool;
  fn named(&self, key: &str) -> Option<&str>;
  fn named_with_loc(&self, key: &str) -> Option<(&str, SourceLocation)>;
  fn ordered_list_custom_number_style(&self) -> Option<&'static str>;
  fn unordered_list_custom_marker_style(&self) -> Option<&'static str>;
  fn block_style(&self) -> Option<BlockContext>;
  fn id(&self) -> Option<&SourceString<'_>>;
  fn roles(&self) -> impl Iterator<Item = &SourceString<'_>>;
}

impl AttrData for AttrList<'_> {
  fn roles(&self) -> impl Iterator<Item = &SourceString<'_>> {
    self.roles.iter().filter(|s| !s.is_empty())
  }

  fn id(&self) -> Option<&SourceString<'_>> {
    self.id.as_ref()
  }

  fn is_empty(&self) -> bool {
    self.positional.is_empty()
      && self.named.0.is_empty()
      && self.id.is_none()
      && self.roles.is_empty()
      && self.options.is_empty()
  }

  fn str_positional_at(&self, index: usize) -> Option<&str> {
    let Some(Some(nodes)) = self.positional.get(index) else {
      return None;
    };
    nodes.single_text()
  }

  fn has_option(&self, option: &str) -> bool {
    self.options.iter().any(|s| s.src == option)
  }

  fn has_str_positional(&self, positional: &str) -> bool {
    self
      .positional
      .iter()
      .enumerate()
      .any(|(i, _)| self.str_positional_at(i) == Some(positional))
  }

  fn is_source(&self) -> bool {
    self.source_language().is_some()
  }

  // TODO: this is incorrect, see https://github.com/jaredh159/asciidork/issues/4
  fn source_language(&self) -> Option<&str> {
    match (self.str_positional_at(0), self.str_positional_at(1)) {
      (None | Some("source"), Some(lang)) => Some(lang),
      _ => None,
    }
  }

  fn has_role(&self, role: &str) -> bool {
    self.roles.iter().any(|s| s.src == role)
  }

  fn named(&self, key: &str) -> Option<&str> {
    self
      .named
      .get(key)
      .and_then(|s| if s.is_empty() { Some("") } else { s.single_text() })
  }

  fn named_with_loc(&self, key: &str) -> Option<(&str, SourceLocation)> {
    self.named.get_with_src(key).and_then(|(src, nodes)| {
      if nodes.is_empty() {
        Some(("", src.loc))
      } else {
        let first = nodes.first().unwrap();
        let last = nodes.last().unwrap();
        let loc = SourceLocation::spanning(first.loc, last.loc);
        nodes.single_text().map(|t| (t, loc))
      }
    })
  }

  /// https://docs.asciidoctor.org/asciidoc/latest/blocks/#block-style
  fn block_style(&self) -> Option<BlockContext> {
    if let Some(first_positional) = self.str_positional_at(0) {
      BlockContext::derive(first_positional)
    } else {
      None
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/lists/unordered/#custom-markers
  fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
    // documented to support these, but seems like in practice
    // they actually pass through ANY first positional attr
    match self.str_positional_at(0) {
      Some("square") => Some("square"),
      Some("circle") => Some("circle"),
      Some("disc") => Some("disc"),
      Some("none") => Some("none"),
      Some("no-bullet") => Some("no-bullet"),
      Some("unstyled") => Some("unstyled"),
      _ => None,
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/lists/ordered/#styles
  fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
    match self.str_positional_at(0) {
      Some("arabic") => Some("arabic"),
      Some("decimal") => Some("decimal"), // html only
      Some("loweralpha") => Some("loweralpha"),
      Some("upperalpha") => Some("upperalpha"),
      Some("lowerroman") => Some("lowerroman"),
      Some("upperroman") => Some("upperroman"),
      Some("lowergreek") => Some("lowergreek"), // html only
      _ => None,
    }
  }
}

impl<'arena> AttrList<'arena> {
  pub fn new(loc: SourceLocation, bump: &'arena Bump) -> Self {
    AttrList {
      positional: BumpVec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: BumpVec::new_in(bump),
      options: BumpVec::new_in(bump),
      loc,
    }
  }

  pub fn take_positional(&mut self, n: usize) -> Option<InlineNodes<'arena>> {
    if self.positional.len() <= n {
      None
    } else {
      self.positional[n].take()
    }
  }

  pub fn insert_named(&mut self, key: SourceString<'arena>, value: InlineNodes<'arena>) {
    self.named.insert(key, value);
    self.positional.push(None);
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'arena>(BumpVec<'arena, (SourceString<'arena>, InlineNodes<'arena>)>);

impl<'arena> Named<'arena> {
  pub fn new_in(bump: &'arena Bump) -> Self {
    Named(BumpVec::new_in(bump))
  }

  pub const fn from(vec: BumpVec<'arena, (SourceString<'arena>, InlineNodes<'arena>)>) -> Self {
    Named(vec)
  }

  fn insert(&mut self, key: SourceString<'arena>, value: InlineNodes<'arena>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&InlineNodes<'arena>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }

  pub fn get_with_src(&self, key: &str) -> Option<(SourceString<'arena>, &InlineNodes<'arena>)> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some((k.clone(), v)) } else { None })
  }
}
