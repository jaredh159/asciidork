use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_list(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
    meta: Option<BlockMetadata<'bmp>>,
  ) -> Result<Block<'bmp>> {
    let mut first_line = lines.consume_current().unwrap();
    first_line.trim_leading_whitespace();
    let marker = first_line
      .current_token()
      .unwrap()
      .to_list_marker()
      .unwrap();
    self.ctx.list_stack.push(marker);
    lines.restore_if_nonempty(first_line);
    self.restore_lines(lines);

    let mut items = BumpVec::new_in(self.bump);
    while let Some(item) = self.parse_list_item()? {
      items.push(item);
    }

    let variant = ListVariant::from(marker);
    let (title, attrs, start) = meta.map(|m| (m.title, m.attrs, m.start)).unwrap_or((
      None,
      None,
      items.first().unwrap().loc_start(),
    ));
    Ok(Block {
      title,
      attrs,
      loc: SourceLocation::new(start, items.last().unwrap().loc_end().unwrap()),
      context: variant.to_context(),
      content: BlockContent::List { variant, items },
    })
  }

  fn parse_list_item(&mut self) -> Result<Option<ListItem<'bmp>>> {
    let Some(mut lines) = self.read_lines() else {
      return Ok(None);
    };
    if !lines.starts_list() {
      self.restore_lines(lines);
      return Ok(None);
    }
    let mut line = lines.consume_current().unwrap();
    let marker = line.consume_to_string_until(Whitespace, self.bump);
    line.discard_assert(Whitespace);

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
    let blocks = self.parse_list_item_blocks(lines)?;

    Ok(Some(ListItem { blocks, marker, principle }))
  }

  fn parse_list_item_blocks(
    &mut self,
    lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<BumpVec<'bmp, Block<'bmp>>> {
    let mut blocks = BumpVec::new_in(self.bump);
    // if we're on a contiguous line, we parse off a block IF:
    //  --> it's a NESTED list item
    //  --> we have a list continuation followed by a block
    if let Some(line) = lines.current() {
      if line.starts_nested_list(&self.ctx.list_stack) {
        println!("starts_nested_list");
        blocks.push(self.parse_list(lines, None)?);
        return Ok(blocks);
      }
    }

    // ELSE IF the next Contiguous Lines starts a NESTED list, parse a block
    // ELSE, we're done

    // this might need to be recursive, and we might need to pass in a &mut blocks vec...

    self.restore_lines(lines);
    Ok(blocks)
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_list_separation() {
    use BlockContext::*;
    let cases: Vec<(&str, &[BlockContext])> = vec![
      ("* foo\n\n[]\n* bar", &[UnorderedList, UnorderedList]),
      ("* foo\n\n\n* bar", &[UnorderedList]),
      ("* foo\n\n\n** bar", &[UnorderedList]),
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
          assert_eq!(
            blocks.len(),
            block_contexts.len(),
            "input was: \n\n```\n{}\n```",
            input
          );
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
        "* foo bar\n  baz",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: b.src("*", l(0, 1)),
          principle: b.inodes([
            n_text("foo bar", 2, 9, b),
            n(Inline::JoiningNewline, l(9, 10)),
            n_text("baz", 12, 15, b),
          ]),
          blocks: b.vec([]),
        }]),
      ),
      (
        "* foo\n** bar",
        BlockContext::UnorderedList,
        b.vec([ListItem {
          marker: b.src("*", l(0, 1)),
          principle: b.inodes([n_text("foo", 2, 5, b)]),
          blocks: b.vec([Block {
            title: None,
            attrs: None,
            content: BlockContent::List {
              variant: ListVariant::Unordered,
              items: b.vec([ListItem {
                marker: b.src("**", l(6, 8)),
                principle: b.inodes([n_text("bar", 9, 12, b)]),
                blocks: b.vec([]),
              }]),
            },
            context: BlockContext::UnorderedList,
            loc: l(6, 12),
          }]),
        }]),
      ),
      (
        "* foo\n* bar",
        BlockContext::UnorderedList,
        b.vec([
          ListItem {
            marker: b.src("*", l(0, 1)),
            principle: b.inodes([n_text("foo", 2, 5, b)]),
            blocks: b.vec([]),
          },
          ListItem {
            marker: b.src("*", l(6, 7)),
            principle: b.inodes([n_text("bar", 8, 11, b)]),
            blocks: b.vec([]),
          },
        ]),
      ),
      (
        ". foo\n\n. bar",
        BlockContext::OrderedList,
        b.vec([
          ListItem {
            marker: b.src(".", l(0, 1)),
            principle: b.inodes([n_text("foo", 2, 5, b)]),
            blocks: b.vec([]),
          },
          ListItem {
            marker: b.src(".", l(7, 8)),
            principle: b.inodes([n_text("bar", 9, 12, b)]),
            blocks: b.vec([]),
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
      assert_eq!(block.context, context, "input was: \n{}\n", input);
      if let BlockContent::List { items, .. } = block.content {
        assert_eq!(items, expected_items, "input was: \n{}\n", input);
      } else {
        panic!("expected list, got {:?}", block.content);
      }
    }
  }
}
