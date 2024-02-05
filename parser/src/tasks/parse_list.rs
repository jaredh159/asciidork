use crate::internal::*;
use crate::variants::token::*;

impl<'bmp, 'src> Parser<'bmp, 'src> {
  pub(crate) fn parse_list(
    &mut self,
    mut lines: ContiguousLines<'bmp, 'src>,
  ) -> Result<(ListVariant, BumpVec<'bmp, ListItem<'bmp>>)> {
    let mut first_line = lines.consume_current().unwrap();
    first_line.discard_leading_whitespace();
    let list_type = first_line.current_token().unwrap().to_list_type().unwrap();
    let mut items = BumpVec::new_in(self.bump);
    lines.restore(first_line);

    while let Some(item) = self.parse_list_item(&mut lines)? {
      items.push(item);
    }

    Ok((list_type, items))
  }

  fn parse_list_item(
    &mut self,
    lines: &mut ContiguousLines<'bmp, 'src>,
  ) -> Result<Option<ListItem<'bmp>>> {
    let Some(mut line) = lines.consume_current() else {
      return Ok(None);
    };
    if !line.starts_list_item() {
      lines.restore(line);
      return Ok(None);
    }
    let marker = line.consume_to_string_until(Whitespace, self.bump);
    line.discard_assert(Whitespace);

    // probably not correct...
    let principle = self.parse_inlines(&mut line.into_lines_in(self.bump))?;
    Ok(Some(ListItem {
      marker,
      principle,
      blocks: BumpVec::new_in(self.bump),
    }))
  }
}

// tests

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn test_parse_simple_list() {
    let input = "* foo\n* bar";
    let b = &Bump::new();
    let mut parser = Parser::new(b, input);
    let lines = parser.read_lines().unwrap();
    let (t, items) = parser.parse_list(lines).unwrap();
    assert_eq!(t, ListVariant::Unordered);
    assert_eq!(
      items,
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
        }
      ])
    );
  }
}
