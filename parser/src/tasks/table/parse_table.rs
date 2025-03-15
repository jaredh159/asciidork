use std::collections::HashSet;
use std::ops::Range;

use bumpalo::collections::CollectIn;

use super::{context::*, DataFormat, TableTokens};
use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_table(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let delim_line = lines.consume_current().unwrap();
    let first_token = delim_line.current_token().unwrap();
    let delim_ch = first_token.lexeme.as_bytes()[0];
    debug_assert!(first_token.lexeme.len() == 1);

    let col_specs = meta
      .attrs
      .named("cols")
      .map(|cols_attr| self.parse_col_specs(cols_attr))
      .unwrap_or_else(|| bvec![in self.bump]);

    let mut format = match (meta.attrs.named("format"), delim_ch) {
      (Some("psv"), _) => DataFormat::Csv('|'),
      (Some("csv"), _) => DataFormat::Csv(','),
      (Some("tsv"), _) => DataFormat::Csv('\t'),
      (Some("dsv"), _) => DataFormat::Delimited(':'),
      (_, b':') => DataFormat::Delimited(':'),
      (_, b',') => DataFormat::Csv(','),
      (_, b'|') => DataFormat::Prefix('|'),
      (_, b'!') => DataFormat::Prefix('!'),
      _ => DataFormat::Prefix(delim_ch as char),
    };

    if let Some(sep) = meta.attrs.named("separator") {
      let msg = "Cell separator must be exactly one character";
      let mut chars = sep.chars();
      match chars.next() {
        None => self.err_at_pattern(msg, meta.start_loc, "separator")?,
        Some(ch) => {
          format.replace_separator(ch);
          if chars.next().is_some() {
            self.err_at_pattern(msg, meta.start_loc, sep)?;
          }
        }
      }
    }

    let col_widths = col_specs
      .iter()
      .map(|spec| spec.width)
      .collect_in::<BumpVec<'_, _>>(self.bump);

    let mut ctx = TableContext {
      delim_ch,
      format,
      cell_separator: format.separator(),
      embeddable_cell_separator: format.embeddable_separator(),
      cell_separator_tokenkind: format.separator_token_kind(),
      num_cols: col_specs.len(),
      counting_cols: col_specs.is_empty(),
      col_specs,
      header_row: HeaderRow::Unknown,
      header_reparse_cells: bvec![in self.bump],
      autowidths: meta.attrs.has_option("autowidth"),
      phantom_cells: HashSet::new(),
      dsv_last_consumed: DsvLastConsumed::Other,
      effective_row_idx: 0,
      table: Table {
        col_widths: col_widths.into(),
        header_row: None,
        rows: bvec![in self.bump],
        footer_row: None,
      },
    };

    if meta.attrs.has_option("header") {
      ctx.header_row = HeaderRow::ExplicitlySet;
    } else if meta.attrs.has_option("noheader") {
      ctx.header_row = HeaderRow::ExplicitlyUnset;
    } else if lines.len() != 1 {
      ctx.header_row = HeaderRow::FoundNone;
    }

    let (mut tokens, end_loc) = self.table_content(lines, &delim_line)?;
    if ctx.counting_cols {
      if !matches!(ctx.format, DataFormat::Prefix(_)) {
        self.parse_dsv_implicit_first_row(&mut tokens, &mut ctx)?;
      } else {
        self.parse_psv_implicit_first_row(&mut tokens, &mut ctx)?;
      }
    }

    if !matches!(ctx.format, DataFormat::Prefix(_)) {
      while let Some(row) = self.parse_dsv_table_row(&mut tokens, &mut ctx)? {
        self.push_table_row(row, &mut ctx)?;
      }
    } else {
      while let Some(row) = self.parse_psv_table_row(&mut tokens, &mut ctx)? {
        self.push_table_row(row, &mut ctx)?;
      }
    }

    if meta.attrs.has_option("footer") && !ctx.table.rows.is_empty() {
      ctx.table.footer_row = Some(ctx.table.rows.pop().unwrap());
    }

    Ok(Block {
      content: BlockContent::Table(ctx.table),
      context: BlockContext::Table,
      meta,
      loc: MultiSourceLocation::spanning(first_token.loc, end_loc),
    })
  }

  pub(crate) fn push_table_row(
    &mut self,
    mut row: Row<'arena>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<()> {
    if ctx.table.rows.is_empty()
      && ctx.table.header_row.is_none()
      && ctx.header_row.known_to_exist()
    {
      if ctx.header_row == HeaderRow::FoundImplicit {
        self.reparse_header_cells(&mut row, ctx)?;
      }
      ctx.table.header_row = Some(row);
    } else {
      ctx.table.rows.push(row);
      if ctx.header_row.is_unknown() {
        ctx.header_row = HeaderRow::FoundNone;
      }
    }
    Ok(())
  }

  pub(crate) fn finish_implicit_header_row(
    &mut self,
    cells: BumpVec<'arena, Cell<'arena>>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<()> {
    if cells.is_empty() {
      return Ok(());
    }
    ctx.effective_row_idx += 1;
    let width = if ctx.autowidths { ColWidth::Auto } else { ColWidth::Proportional(1) };
    ctx.table.col_widths = ColWidths::new(bvec![in self.bump; width; ctx.num_cols]);
    if ctx.header_row.known_to_exist() {
      let mut row = Row::new(cells);
      if ctx.header_row == HeaderRow::FoundImplicit {
        self.reparse_header_cells(&mut row, ctx)?;
      }
      ctx.table.header_row = Some(row);
    } else {
      ctx.table.rows.push(Row::new(cells));
      if ctx.header_row.is_unknown() {
        ctx.header_row = HeaderRow::FoundNone;
      }
    }
    Ok(())
  }

  pub(crate) fn finish_cell(
    &mut self,
    cell_spec: CellSpec,
    mut cell_tokens: Line<'arena>,
    col_index: usize,
    ctx: &mut TableContext<'arena>,
    mut loc: Range<u32>,
  ) -> Result<Option<(Cell<'arena>, u8)>> {
    let col_spec = ctx.col_specs.get(col_index);
    let mut cell_style = cell_spec
      .style
      .unwrap_or_else(|| col_spec.map_or(CellContentStyle::Default, |col| col.style));

    if ctx.header_row.known_to_exist() && ctx.table.header_row.is_none() {
      cell_style = CellContentStyle::Default;
    }

    let mut trimmed_implicit_header = false;
    if ctx.header_row.is_unknown() && matches!(ctx.format, DataFormat::Prefix(_)) {
      let mut ws = SmallVec::<[TokenKind; 12]>::new();
      while cell_tokens.last().is_whitespaceish() {
        let token = cell_tokens.pop().unwrap();
        loc.end = token.loc.start;
        ws.push(token.kind);
      }
      if ws.len() > 1 && ws[ws.len() - 2..] == [Newline, Newline] {
        trimmed_implicit_header = true;
        ctx.header_row = HeaderRow::FoundImplicit;
      }
    } else {
      while cell_tokens.last().is_whitespaceish() {
        loc.end = cell_tokens.pop().unwrap().loc.start;
      }
    }

    let repeat = cell_spec.duplication.unwrap_or(1);
    if cell_style == CellContentStyle::AsciiDoc {
      if ctx.header_row.is_unknown() || trimmed_implicit_header {
        ctx.header_reparse_cells.push(ParseCellData {
          cell_tokens: cell_tokens.clone(),
          cell_spec: cell_spec.clone(),
          col_spec: col_spec.cloned(),
        });
      }
      cell_tokens.trim_for_cell(cell_style);
      cell_tokens.remove_resolved_attr_refs();
      let cell_parser = self.cell_parser(cell_tokens.into_bytes(), loc.start);
      return match cell_parser.parse() {
        Ok(ParseResult { document, warnings }) => {
          if !warnings.is_empty() {
            self.errors.borrow_mut().extend(warnings);
          }
          let content = CellContent::AsciiDoc(document);
          let cell = Cell::new(content, cell_spec, col_spec.cloned());
          Ok(Some((cell, repeat)))
        }
        Err(mut diagnostics) => {
          if !diagnostics.is_empty() && self.strict {
            Err(diagnostics.remove(0))
          } else {
            self.errors.borrow_mut().extend(diagnostics);
            Ok(None)
          }
        }
      };
    }

    let cell_data = ParseCellData {
      cell_tokens,
      cell_spec,
      col_spec: col_spec.cloned(),
    };
    if cell_style == CellContentStyle::Literal
      && (ctx.header_row.is_unknown() || trimmed_implicit_header)
    {
      ctx.header_reparse_cells.push(cell_data.clone());
    }
    let cell = self.parse_non_asciidoc_cell(cell_data, cell_style)?;
    Ok(Some((cell, repeat)))
  }

  // header cells don't have a style, but we don't always know
  // we have an implicit header until we've parsed too far, so
  // we come back and modify the cells after we discover an implicit
  // header - for asciidoc and literal this means reparsing the tokens
  // but for other styles we can just re-wrap the nodes
  // asciidoctor does a reparse of header cells for this reason as well,
  // see: https://github.com/asciidoctor/asciidoctor/commit/ca2ca428
  fn reparse_header_cells(
    &mut self,
    row: &mut Row<'arena>,
    ctx: &mut TableContext<'arena>,
  ) -> Result<()> {
    for idx in 0..row.cells.len() {
      let mut content = CellContent::Literal(InlineNodes::new(self.bump));
      std::mem::swap(&mut row.cells[idx].content, &mut content);
      row.cells[idx].content = match content {
        CellContent::AsciiDoc(_) | CellContent::Literal(_) => {
          let data = ctx.header_reparse_cells.remove(0);
          let cell = self.parse_non_asciidoc_cell(data, CellContentStyle::Default)?;
          cell.content
        }
        CellContent::Emphasis(paras) => CellContent::Default(paras),
        CellContent::Header(paras) => CellContent::Default(paras),
        CellContent::Monospace(paras) => CellContent::Default(paras),
        CellContent::Strong(paras) => CellContent::Default(paras),
        content => content,
      }
    }
    Ok(())
  }

  fn parse_non_asciidoc_cell(
    &mut self,
    mut data: ParseCellData<'arena>,
    cell_style: CellContentStyle,
  ) -> Result<Cell<'arena>> {
    let nodes = if data.cell_tokens.is_empty() {
      InlineNodes::new(self.bump)
    } else {
      data.cell_tokens.trim_for_cell(cell_style);
      let prev_tbl_ctx = self.ctx.table_cell_ctx;
      self.ctx.table_cell_ctx = TableCellContext::Cell;
      let prev_subs = self.ctx.subs;
      self.ctx.subs = cell_style.into();
      let inlines = self.parse_inlines(&mut data.cell_tokens.into_lines())?;
      self.ctx.subs = prev_subs;
      self.ctx.table_cell_ctx = prev_tbl_ctx;
      inlines
    };

    let content = match cell_style {
      CellContentStyle::Default => CellContent::Default(self.split_paragraphs(nodes)),
      CellContentStyle::Emphasis => CellContent::Emphasis(self.split_paragraphs(nodes)),
      CellContentStyle::Header => CellContent::Header(self.split_paragraphs(nodes)),
      CellContentStyle::Monospace => CellContent::Monospace(self.split_paragraphs(nodes)),
      CellContentStyle::Strong => CellContent::Strong(self.split_paragraphs(nodes)),
      CellContentStyle::Literal => CellContent::Literal(nodes),
      CellContentStyle::AsciiDoc => unreachable!("Parser::finish_cell() asciidoc"),
    };
    Ok(Cell::new(content, data.cell_spec, data.col_spec))
  }

  fn split_paragraphs(&self, nodes: InlineNodes<'arena>) -> BumpVec<'arena, InlineNodes<'arena>> {
    let mut paras = bvec![in self.bump];
    if nodes.is_empty() {
      return paras;
    }
    let mut index = 0;
    paras.push(InlineNodes::new(self.bump));
    for node in nodes.into_vec() {
      if matches!(node.content, Inline::Newline)
        && paras[index]
          .last()
          .is_some_and(|n| n.content == Inline::Newline)
      {
        paras[index].pop();
        index += 1;
        paras.push(InlineNodes::new(self.bump));
      } else {
        paras[index].push(node);
      }
    }
    paras
  }

  fn table_content(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    start_delim: &Line<'arena>,
  ) -> Result<(TableTokens<'arena>, SourceLocation)> {
    let mut tokens = Deq::with_capacity(lines.num_tokens(), self.bump);
    let delim_loc = start_delim.last_loc().unwrap();
    let mut end = delim_loc.end + 1;
    while let Some(line) = lines.consume_current() {
      // TODO(perf): src_eq is O(n), see if we can refactor
      if line.src_eq(start_delim) {
        self.restore_lines(lines);
        return Ok((TableTokens::new(tokens), line.last_loc().unwrap()));
      }
      if line.is_comment() {
        continue;
      }
      if let Some(loc) = line.last_loc() {
        end = loc.end;
      }
      line.drain_into(&mut tokens);
      if !lines.is_empty() {
        tokens.push(newline_token(end, self.bump));
        end += 1;
      }
    }
    while let Some(next_line) = self.read_line()? {
      if next_line.is_comment() {
        continue;
      }
      if !tokens.is_empty() {
        tokens.push(newline_token(end, self.bump));
        end += 1;
      }
      // TODO(perf): src_eq is O(n), see if we can refactor
      if next_line.src_eq(start_delim) {
        return Ok((TableTokens::new(tokens), next_line.last_loc().unwrap()));
      }
      if let Some(loc) = next_line.last_loc() {
        end = loc.end;
      }
      next_line.drain_into(&mut tokens);
    }
    self.err_line("Table never closed, started here", start_delim)?;
    Ok((TableTokens::new(tokens), self.lexer.loc()))
  }
}

fn newline_token(start: u32, bump: &Bump) -> Token {
  Token {
    kind: TokenKind::Newline,
    lexeme: BumpString::from_str_in("\n", bump),
    loc: SourceLocation::new(start, start + 1),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  assert_error!(
    multichar_cell_separator,
    adoc! {r#"
      [separator="||"]
      |===
      ||one||two
      |===
    "# },
    error! { r#"
       --> test.adoc:1:13
        |
      1 | [separator="||"]
        |             ^^ Cell separator must be exactly one character
    "#}
  );

  assert_error!(
    empty_cell_separator,
    adoc! {r#"
      [separator=""]
      |===
      ||one||two
      |===
    "# },
    error! { r#"
       --> test.adoc:1:2
        |
      1 | [separator=""]
        |  ^^^^^^^^^ Cell separator must be exactly one character
    "#}
  );
}
