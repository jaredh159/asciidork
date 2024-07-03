#![allow(dead_code)]

use std::ops::Range;

use asciidork_ast::prelude::*;
use test_utils::*;

pub fn named(
  pairs: &[(&'static str, Range<u32>, &'static str, Range<u32>)],
) -> asciidork_ast::AttrList<'static> {
  let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
  attrs.loc.start = pairs[0].1.start - 1;
  attrs.loc.end = pairs[pairs.len() - 1].3.end + 1;
  for (name, name_range, value, value_range) in pairs {
    attrs.named.insert(
      src!(name, name_range.clone()),
      just!(value, value_range.clone()),
    );
  }
  attrs
}

pub fn pos(text: &'static str, range: Range<u32>) -> asciidork_ast::AttrList<'static> {
  let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
  attrs.loc.start = range.start - 1;
  attrs.loc.end = range.end + 1;
  attrs.positional.push(Some(nodes![node!(text; range)]));
  attrs
}

pub fn role(text: &'static str, range: Range<u32>) -> asciidork_ast::AttrList<'static> {
  let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
  attrs.loc.start = range.start - 2;
  attrs.loc.end = range.end + 1;
  attrs.roles.push(src!(text, range));
  attrs
}

pub fn opt(text: &'static str, range: Range<u32>) -> asciidork_ast::AttrList<'static> {
  let mut attrs = AttrList::new(SourceLocation::new(0, 0), leaked_bump());
  attrs.loc.start = range.start - 2;
  attrs.loc.end = range.end + 1;
  attrs.options.push(src!(text, range));
  attrs
}
