use crate::internal::*;

// https://docs.asciidoctor.org/asciidoc/latest/attributes/positional-and-named-attributes/
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AttrList<'bmp> {
  pub positional: BumpVec<'bmp, Option<InlineNodes<'bmp>>>,
  pub named: Named<'bmp>,
  pub id: Option<SourceString<'bmp>>,
  pub roles: BumpVec<'bmp, SourceString<'bmp>>,
  pub options: BumpVec<'bmp, SourceString<'bmp>>,
  pub loc: SourceLocation,
}

impl<'bmp> AttrList<'bmp> {
  pub fn new(loc: SourceLocation, bump: &'bmp Bump) -> Self {
    AttrList {
      positional: BumpVec::new_in(bump),
      named: Named::new_in(bump),
      id: None,
      roles: BumpVec::new_in(bump),
      options: BumpVec::new_in(bump),
      loc,
    }
  }

  /// https://docs.asciidoctor.org/asciidoc/latest/blocks/#block-style
  pub fn block_style(&self) -> Option<BlockContext> {
    if let Some(first_positional) = self.str_positional_at(0) {
      BlockContext::derive(first_positional)
    } else {
      None
    }
  }

  // https://docs.asciidoctor.org/asciidoc/latest/lists/unordered/#custom-markers
  pub fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
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
  pub fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
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

  pub fn named(&self, key: &str) -> Option<&str> {
    self.named.get(key).map(|s| s.src.as_str())
  }

  pub fn str_positional_at(&self, index: usize) -> Option<&str> {
    let Some(Some(nodes)) = self.positional.get(index) else {
      return None;
    };
    if nodes.len() != 1 {
      return None;
    }
    let Inline::Text(positional) = &nodes[0].content else {
      return None;
    };
    Some(positional.as_str())
  }

  pub fn has_option(&self, option: &str) -> bool {
    self.options.iter().any(|s| s.src == option)
  }

  pub fn has_str_positional(&self, positional: &str) -> bool {
    self
      .positional
      .iter()
      .enumerate()
      .any(|(i, _)| self.str_positional_at(i) == Some(positional))
  }

  pub fn is_source(&self) -> bool {
    self.source_language().is_some()
  }

  // TODO: this is incorrect, see https://github.com/jaredh159/asciidork/issues/4
  pub fn source_language(&self) -> Option<&str> {
    match (self.str_positional_at(0), self.str_positional_at(1)) {
      (None | Some("source"), Some(lang)) => Some(lang),
      _ => None,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Named<'bmp>(BumpVec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>);

impl<'bmp> Named<'bmp> {
  pub fn new_in(bump: &'bmp Bump) -> Self {
    Named(BumpVec::new_in(bump))
  }

  pub fn from(vec: BumpVec<'bmp, (SourceString<'bmp>, SourceString<'bmp>)>) -> Self {
    Named(vec)
  }

  pub fn insert(&mut self, key: SourceString<'bmp>, value: SourceString<'bmp>) {
    self.0.push((key, value));
  }

  pub fn get(&self, key: &str) -> Option<&SourceString<'bmp>> {
    self
      .0
      .iter()
      .find_map(|(k, v)| if k == key { Some(v) } else { None })
  }
}
