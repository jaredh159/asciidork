use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_list(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: Option<BlockMetadata<'bmp>>,
  ) -> Result<Block<'bmp>> {
    let first_line = lines.consume_current().unwrap();
    let marker = first_line.list_marker().unwrap();
    let variant = ListVariant::from(marker);
    lines.restore_if_nonempty(first_line);
    self.restore_lines(lines);

    self.ctx.list.stack.push(marker);
    let depth = self.ctx.list.stack.depth();
    let mut items = BumpVec::new_in(self.bump);
    while let Some(item) = self.parse_list_item(variant)? {
      items.push(item);
    }
    self.ctx.list.stack.pop();

    let (title, attrs, start) = meta.map(|m| (m.title, m.attrs, m.start)).unwrap_or((
      None,
      None,
      items.first().unwrap().loc_start(),
    ));
    Ok(Block {
      title,
      attrs,
      loc: SourceLocation::new(start, items.last().unwrap().last_loc_end().unwrap()),
      context: variant.to_context(),
      content: BlockContent::List { variant, depth, items },
    })
  }

  fn parse_list_item(&mut self, list_variant: ListVariant) -> Result<Option<ListItem<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };

    let Some(marker) = lines.first().and_then(|line| line.list_marker()) else {
      self.restore_lines(lines);
      return Ok(None);
    };

    if !self.ctx.list.stack.continues_current_list(marker) {
      self.restore_lines(lines);
      return Ok(None);
    }

    let mut line = lines.consume_current().unwrap();
    line.trim_leading_whitespace();
    if list_variant == ListVariant::Description {
      return self.parse_description_list_item(marker, line, lines);
    }

    let marker_src = line.consume_to_string_until(Whitespace, self.bump);
    line.discard_assert(Whitespace);
    let checklist = if list_variant == ListVariant::Unordered {
      line.consume_checklist_item(self.bump)
    } else {
      None
    };

    let mut item_lines = bvec![in self.bump; line];
    while lines
      .current()
      .map(|line| line.continues_list_item_principle())
      .unwrap_or(false)
    {
      let mut line = lines.consume_current().unwrap();
      line.discard_leading_whitespace();
      item_lines.push(line);
    }

    let mut item_lines = ContiguousLines::new(item_lines);
    let principle = self.parse_inlines(&mut item_lines)?;
    let blocks = self.parse_list_item_blocks(lines, BumpVec::new_in(self.bump))?;

    Ok(Some(ListItem {
      blocks,
      marker,
      marker_src,
      checklist,
      principle,
    }))
  }

  fn parse_list_item_blocks(
    &mut self,
    lines: ContiguousLines<'bmp, 'src>,
    mut blocks: BumpVec<'bmp, Block<'bmp>>,
  ) -> Result<BumpVec<'bmp, Block<'bmp>>> {
    if lines.starts_nested_list(&self.ctx.list.stack, true) {
      self.restore_lines(lines);
      blocks.push(self.parse_block(None)?.unwrap());
      return Ok(blocks);
    }

    if lines.starts_list_continuation() {
      self.restore_lines(lines);
      return self.parse_list_continuation_blocks(blocks);
    }

    if !lines.is_empty() {
      self.restore_lines(lines);
      return Ok(blocks);
    }

    let Some(lines) = self.read_lines() else {
      return Ok(blocks);
    };

    // ELSE IF the next Contiguous Lines starts a NESTED list, parse a block
    if lines.starts_nested_list(&self.ctx.list.stack, false) {
      blocks.push(self.parse_list(lines, None)?);
      return Ok(blocks);
    }

    self.restore_lines(lines);
    Ok(blocks)
  }

  fn parse_list_continuation_blocks(
    &mut self,
    mut accum: BumpVec<'bmp, Block<'bmp>>,
  ) -> Result<BumpVec<'bmp, Block<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(accum);
    };
    if !lines.starts_list_continuation() {
      self.restore_lines(lines);
      return Ok(accum);
    }
    lines.consume_current();
    self.restore_lines(lines);
    self.ctx.list.parsing_continuations = true;
    accum.push(self.parse_block(None)?.unwrap());
    self.ctx.list.parsing_continuations = false;
    self.parse_list_continuation_blocks(accum)
  }

  pub fn parse_description_list_item(
    &mut self,
    marker: ListMarker,
    mut line: Line<'bmp, 'src>,
    mut lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<Option<ListItem<'bmp>>> {
    let principle = {
      let before_delim = line.extract_line_before(TermDelimiter, self.bump);
      self.parse_inlines(&mut before_delim.into_lines_in(self.bump))?
    };

    let marker_token = line.consume_current().unwrap();
    let marker_src = marker_token.to_source_string(self.bump);

    line.trim_leading_whitespace();
    lines.restore_if_nonempty(line);
    let blocks = self.parse_description_list_item_blocks(lines)?;

    Ok(Some(ListItem {
      blocks,
      marker,
      marker_src,
      checklist: None,
      principle,
    }))
  }

  pub fn parse_description_list_item_blocks(
    &mut self,
    lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<BumpVec<'bmp, Block<'bmp>>> {
    self.restore_lines(lines);
    let mut blocks = BumpVec::new_in(self.bump);
    if let Some(block) = self.parse_block(None)? {
      blocks.push(block);
    }
    let Some(lines) = self.read_lines() else {
      return Ok(blocks);
    };
    if lines.starts_list_continuation() {
      self.restore_lines(lines);
      return self.parse_list_continuation_blocks(blocks);
    }
    self.restore_lines(lines);
    Ok(blocks)
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use test_utils::{adoc, assert_eq, parse_list};

  #[test]
  fn test_simple_description_list() {
    let input = "foo:: bar";
    parse_list!(input, list, bump);
    assert_list(
      input,
      list,
      BlockContext::DescriptionList,
      &[ListItem {
        marker: ListMarker::Colons(2),
        marker_src: bump.src("::", l(3, 5)),
        principle: bump.inodes([n_text("foo", 0, 3, bump)]),
        checklist: None,
        blocks: bump.vec([Block {
          title: None,
          attrs: None,
          content: BlockContent::Simple(bump.inodes([n_text("bar", 6, 9, bump)])),
          context: BlockContext::Paragraph,
          loc: l(6, 9),
        }]),
      }],
    );
  }

  #[test]
  fn test_two_line_description_list() {
    let input = "foo::\nbar";
    parse_list!(input, list, bump);
    assert_list(
      input,
      list,
      BlockContext::DescriptionList,
      &[ListItem {
        marker: ListMarker::Colons(2),
        marker_src: bump.src("::", l(3, 5)),
        principle: bump.inodes([n_text("foo", 0, 3, bump)]),
        checklist: None,
        blocks: bump.vec([Block {
          title: None,
          attrs: None,
          content: BlockContent::Simple(bump.inodes([n_text("bar", 6, 9, bump)])),
          context: BlockContext::Paragraph,
          loc: l(6, 9),
        }]),
      }],
    );
  }

  #[test]
  fn test_list_separation() {
    use BlockContext::*;
    let cases: Vec<(&str, &[BlockContext])> = vec![
      ("* foo\n\n[]\n* bar", &[UnorderedList, UnorderedList]),
      ("* foo\n\n\n* bar", &[UnorderedList]),
      ("* foo\n\n\n** bar", &[UnorderedList]),
      ("[square]\n* foo\n[circle]\n** bar", &[UnorderedList]),
      (
        "* foo\n\n//-\n\n. bar",
        &[UnorderedList, Comment, OrderedList],
      ),
      (
        "* foo\n\n//-\n\n* bar",
        &[UnorderedList, Comment, UnorderedList],
      ),
    ];

    let bump = &Bump::new();
    for (input, block_contexts) in cases {
      let parser = Parser::new(bump, input);
      let content = parser.parse().unwrap().document.content;
      match content {
        DocContent::Blocks(blocks) => {
          assert_eq!( blocks.len(), block_contexts.len(), from: input);
          for (block, context) in blocks.iter().zip(block_contexts.iter()) {
            assert_eq!(block.context, *context);
          }
        }
        _ => panic!("expected blocks, got {:?}", content),
      }
    }
  }

  #[test]
  fn test_parse_lists() {
    let b = &Bump::new();
    let cases = vec![
      (
        "* one\n** two\n* one again",
        BlockContext::UnorderedList,
        b.vec([
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(0, 1)),
            principle: b.inodes([n_text("one", 2, 5, b)]),
            checklist: None,
            blocks: b.vec([Block {
              title: None,
              attrs: None,
              content: BlockContent::List {
                variant: ListVariant::Unordered,
                depth: 2,
                items: b.vec([ListItem {
                  marker: ListMarker::Star(2),
                  marker_src: b.src("**", l(6, 8)),
                  principle: b.inodes([n_text("two", 9, 12, b)]),
                  checklist: None,
                  blocks: b.vec([]),
                }]),
              },
              context: BlockContext::UnorderedList,
              loc: l(6, 12),
            }]),
          },
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(13, 14)),
            principle: b.inodes([n_text("one again", 15, 24, b)]),
            checklist: None,
            blocks: b.vec([]),
          },
        ]),
      ),
      (
        "* foo bar\n  baz",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          checklist: None,
          principle: b.inodes([
            n_text("foo bar", 2, 9, b),
            n(Inline::JoiningNewline, l(9, 10)),
            n_text("baz", 12, 15, b),
          ]),
          blocks: b.vec([]),
        }]),
      ),
      (
        "* foo\n[circles]\n** bar",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          principle: b.inodes([n_text("foo", 2, 5, b)]),
          checklist: None,
          blocks: b.vec([Block {
            title: None,
            attrs: Some(AttrList::positional("circles", l(7, 14), b)),
            content: BlockContent::List {
              depth: 2,
              variant: ListVariant::Unordered,
              items: b.vec([ListItem {
                marker: ListMarker::Star(2),
                marker_src: b.src("**", l(16, 18)),
                principle: b.inodes([n_text("bar", 19, 22, b)]),
                checklist: None,
                blocks: b.vec([]),
              }]),
            },
            context: BlockContext::UnorderedList,
            loc: l(6, 22),
          }]),
        }]),
      ),
      (
        "* foo\n** bar",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          principle: b.inodes([n_text("foo", 2, 5, b)]),
          checklist: None,
          blocks: b.vec([Block {
            title: None,
            attrs: None,
            content: BlockContent::List {
              depth: 2,
              variant: ListVariant::Unordered,
              items: b.vec([ListItem {
                marker: ListMarker::Star(2),
                marker_src: b.src("**", l(6, 8)),
                principle: b.inodes([n_text("bar", 9, 12, b)]),
                blocks: b.vec([]),
                checklist: None,
              }]),
            },
            context: BlockContext::UnorderedList,
            loc: l(6, 12),
          }]),
        }]),
      ),
      (
        "* foo\n\n\n** bar",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          principle: b.inodes([n_text("foo", 2, 5, b)]),
          checklist: None,
          blocks: b.vec([Block {
            title: None,
            attrs: None,
            content: BlockContent::List {
              depth: 2,
              variant: ListVariant::Unordered,
              items: b.vec([ListItem {
                checklist: None,
                marker: ListMarker::Star(2),
                marker_src: b.src("**", l(8, 10)),
                principle: b.inodes([n_text("bar", 11, 14, b)]),
                blocks: b.vec([]),
              }]),
            },
            context: BlockContext::UnorderedList,
            loc: l(8, 14),
          }]),
        }]),
      ),
      (
        "* foo\n* bar",
        BlockContext::UnorderedList,
        b.vec([
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(0, 1)),
            principle: b.inodes([n_text("foo", 2, 5, b)]),
            checklist: None,
            blocks: b.vec([]),
          },
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(6, 7)),
            principle: b.inodes([n_text("bar", 8, 11, b)]),
            checklist: None,
            blocks: b.vec([]),
          },
        ]),
      ),
      (
        ". foo\n\n. bar",
        BlockContext::OrderedList,
        b.vec([
          ListItem {
            marker: ListMarker::Dot(1),
            marker_src: b.src(".", l(0, 1)),
            principle: b.inodes([n_text("foo", 2, 5, b)]),
            checklist: None,
            blocks: b.vec([]),
          },
          ListItem {
            marker: ListMarker::Dot(1),
            marker_src: b.src(".", l(7, 8)),
            principle: b.inodes([n_text("bar", 9, 12, b)]),
            checklist: None,
            blocks: b.vec([]),
          },
        ]),
      ),
      (
        adoc! {"
          . Linux

            * Fedora
            * Ubuntu
        "},
        BlockContext::OrderedList,
        b.vec([ListItem {
          marker: ListMarker::Dot(1),
          marker_src: b.src(".", l(0, 1)),
          principle: b.inodes([n_text("Linux", 2, 7, b)]),
          checklist: None,
          blocks: b.vec([Block {
            title: None,
            attrs: None,
            content: BlockContent::List {
              depth: 2,
              variant: ListVariant::Unordered,
              items: b.vec([
                ListItem {
                  marker: ListMarker::Star(1),
                  marker_src: b.src("*", l(11, 12)),
                  principle: b.inodes([n_text("Fedora", 13, 19, b)]),
                  checklist: None,
                  blocks: b.vec([]),
                },
                ListItem {
                  marker: ListMarker::Star(1),
                  marker_src: b.src("*", l(22, 23)),
                  principle: b.inodes([n_text("Ubuntu", 24, 30, b)]),
                  checklist: None,
                  blocks: b.vec([]),
                },
              ]),
            },
            context: BlockContext::UnorderedList,
            loc: l(11, 30),
          }]),
        }]),
      ),
      (
        adoc! {"
          * [*] checked
          * [x] also checked
          * [ ] not checked
        "},
        BlockContext::UnorderedList,
        b.vec([
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(0, 1)),
            checklist: Some((true, b.src("[*]", l(2, 5)))),
            principle: b.inodes([n_text(" checked", 5, 13, b)]),
            blocks: b.vec([]),
          },
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(14, 15)),
            checklist: Some((true, b.src("[x]", l(16, 19)))),
            principle: b.inodes([n_text(" also checked", 19, 32, b)]),
            blocks: b.vec([]),
          },
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(33, 34)),
            checklist: Some((false, b.src("[ ]", l(35, 38)))),
            principle: b.inodes([n_text(" not checked", 38, 50, b)]),
            blocks: b.vec([]),
          },
        ]),
      ),
      (
        adoc! {"
          * principle
          +
          with continuation
        "},
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          checklist: None,
          principle: b.inodes([n_text("principle", 2, 11, b)]),
          blocks: b.vec([Block {
            title: None,
            attrs: None,
            content: BlockContent::Simple(b.inodes([n_text("with continuation", 14, 31, b)])),
            context: BlockContext::Paragraph,
            loc: l(14, 31),
          }]),
        }]),
      ),
      (
        adoc! {"
          * principle
          +
          with continuation
          +
          and another
        "},
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: ListMarker::Star(1),
          marker_src: b.src("*", l(0, 1)),
          checklist: None,
          principle: b.inodes([n_text("principle", 2, 11, b)]),
          blocks: b.vec([
            Block {
              title: None,
              attrs: None,
              content: BlockContent::Simple(b.inodes([n_text("with continuation", 14, 31, b)])),
              context: BlockContext::Paragraph,
              loc: l(14, 31),
            },
            Block {
              title: None,
              attrs: None,
              content: BlockContent::Simple(b.inodes([n_text("and another", 34, 45, b)])),
              context: BlockContext::Paragraph,
              loc: l(34, 45),
            },
          ]),
        }]),
      ),
      (
        adoc! {"
          * principle
          +
          ----
          listing 1
          ----
          +
          some more principle

          * second principle
          +
          ----
          listing 2
          ----
        "},
        BlockContext::UnorderedList,
        b.vec([
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(0, 1)),
            checklist: None,
            principle: b.inodes([n_text("principle", 2, 11, b)]),
            blocks: b.vec([
              Block {
                title: None,
                attrs: None,
                content: BlockContent::Simple(b.inodes([n_text("listing 1", 19, 28, b)])),
                context: BlockContext::Listing,
                loc: l(14, 33),
              },
              Block {
                title: None,
                attrs: None,
                content: BlockContent::Simple(b.inodes([n_text("some more principle", 36, 55, b)])),
                context: BlockContext::Paragraph,
                loc: l(36, 55),
              },
            ]),
          },
          ListItem {
            marker: ListMarker::Star(1),
            marker_src: b.src("*", l(57, 58)),
            checklist: None,
            principle: b.inodes([n_text("second principle", 59, 75, b)]),
            blocks: b.vec([Block {
              title: None,
              attrs: None,
              content: BlockContent::Simple(b.inodes([n_text("listing 2", 83, 92, b)])),
              context: BlockContext::Listing,
              loc: l(78, 97),
            }]),
          },
        ]),
      ),
    ];
    run(cases, b);
  }

  fn run<'bmp>(cases: Vec<(&str, BlockContext, BumpVec<'bmp, ListItem<'bmp>>)>, bump: &Bump) {
    for (input, context, expected_items) in cases {
      let mut parser = Parser::new(bump, input);
      let lines = parser.read_lines().unwrap();
      let block = parser.parse_list(lines, None).unwrap();
      assert_list(input, block, context, &expected_items);
    }
  }

  fn assert_list(
    input: &str,
    block: Block<'_>,
    expected_context: BlockContext,
    expected_items: &[ListItem<'_>],
  ) {
    assert_eq!(block.context, expected_context, from: input);
    if let BlockContent::List { items, .. } = block.content {
      assert_eq!(items, expected_items, from: input);
    } else {
      panic!("expected list, got {:?}", block.content);
    }
  }
}
