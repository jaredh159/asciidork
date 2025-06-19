use std::fmt::{Debug, Formatter};

use crate::internal::*;

pub struct ParseResult<'arena> {
  pub document: Document<'arena>,
  pub warnings: Vec<Diagnostic>,
  pub(crate) attr_locs: Vec<(SourceLocation, bool)>,
  pub include_resolver: Option<Box<dyn IncludeResolver>>,
  #[cfg(feature = "attr_ref_observation")]
  pub attr_ref_observer: Option<Box<dyn AttrRefObserver>>,
  lexer: Lexer<'arena>,
}

impl ParseResult<'_> {
  pub fn line_number_with_offset(&self, loc: SourceLocation) -> (u32, u32) {
    self.lexer.line_number_with_offset(loc)
  }

  pub fn source_file_at(&self, idx: u16) -> &SourceFile {
    self.lexer.source_file_at(idx)
  }

  pub fn take_include_resolver_as<T: 'static>(&mut self) -> Option<T> {
    let resolver = self.include_resolver.take()?;
    let any_resolver = resolver as Box<dyn std::any::Any>;
    Some(*any_resolver.downcast::<T>().unwrap())
  }

  #[cfg(feature = "attr_ref_observation")]
  pub fn take_attr_ref_observer_as<T: 'static>(&mut self) -> Option<T> {
    let observer = self.attr_ref_observer.take()?;
    let any_observer = observer as Box<dyn std::any::Any>;
    Some(*any_observer.downcast::<T>().unwrap())
  }
}

impl<'arena> From<Parser<'arena>> for ParseResult<'arena> {
  fn from(parser: Parser<'arena>) -> Self {
    ParseResult {
      document: parser.document,
      warnings: parser.errors.into_inner(),
      attr_locs: parser.attr_locs,
      include_resolver: parser.include_resolver,
      #[cfg(feature = "attr_ref_observation")]
      attr_ref_observer: parser.attr_ref_observer,
      lexer: parser.lexer,
    }
  }
}

impl Debug for ParseResult<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ParseResult")
      .field("document", &self.document)
      .field("warnings", &self.warnings)
      .finish()
  }
}
