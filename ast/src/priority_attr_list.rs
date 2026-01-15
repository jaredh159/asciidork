use crate::internal::*;

pub struct PriorityAttrList<'arena> {
  priority: &'arena AttrList<'arena>,
  rest: &'arena MultiAttrList<'arena>,
}

impl PriorityAttrList<'_> {
  pub const fn new<'a>(
    priority: &'a AttrList<'a>,
    rest: &'a MultiAttrList<'a>,
  ) -> PriorityAttrList<'a> {
    PriorityAttrList { priority, rest }
  }
}

impl AttrData for PriorityAttrList<'_> {
  fn is_empty(&self) -> bool {
    self.priority.is_empty() && self.rest.is_empty()
  }

  fn str_positional_at(&self, index: usize) -> Option<&str> {
    self
      .priority
      .str_positional_at(index)
      .or_else(|| self.rest.str_positional_at(index))
  }

  fn positional_at(&self, index: usize) -> Option<&InlineNodes<'_>> {
    self
      .priority
      .positional_at(index)
      .or_else(|| self.rest.positional_at(index))
  }

  fn has_option(&self, option: &str) -> bool {
    self.priority.has_option(option) || self.rest.has_option(option)
  }

  fn has_str_positional(&self, positional: &str) -> bool {
    self.priority.has_str_positional(positional) || self.rest.has_str_positional(positional)
  }

  fn is_source(&self) -> bool {
    self.priority.is_source() || self.rest.is_source()
  }

  fn source_language(&self) -> Option<&str> {
    self
      .priority
      .source_language()
      .or_else(|| self.rest.source_language())
  }

  fn has_role(&self, role: &str) -> bool {
    self.priority.has_role(role) || self.rest.has_role(role)
  }

  fn named(&self, key: &str) -> Option<&str> {
    self.priority.named(key).or_else(|| self.rest.named(key))
  }

  fn named_with_loc(&self, key: &str) -> Option<(&str, SourceLocation)> {
    self
      .priority
      .named_with_loc(key)
      .or_else(|| self.rest.named_with_loc(key))
  }

  fn ordered_list_custom_number_style(&self) -> Option<&'static str> {
    self
      .priority
      .ordered_list_custom_number_style()
      .or_else(|| self.rest.ordered_list_custom_number_style())
  }

  fn unordered_list_custom_marker_style(&self) -> Option<&'static str> {
    self
      .priority
      .unordered_list_custom_marker_style()
      .or_else(|| self.rest.unordered_list_custom_marker_style())
  }

  fn block_style(&self, context: BlockContext) -> Option<BlockContext> {
    self
      .priority
      .block_style(context)
      .or_else(|| self.rest.block_style(context))
  }

  fn id(&self) -> Option<&SourceString<'_>> {
    self.priority.id().or_else(|| self.rest.id())
  }

  fn roles(&self) -> impl Iterator<Item = &SourceString<'_>> {
    self.priority.roles().chain(self.rest.roles())
  }
}
