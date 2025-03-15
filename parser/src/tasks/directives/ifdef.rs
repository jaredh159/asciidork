use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_ifdef_directive(
    &mut self,
    defined: bool,
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    let src = line.reassemble_src();
    let Some(captures) = regx::DIRECTIVE_IFDEF.captures(&src) else {
      return Ok(DirectiveAction::Passthrough);
    };

    let attrs = captures.get(1).unwrap().as_str();
    let embedded_line = captures.get(2).unwrap();
    self.ctx.ifdef_stack.push(self.string(attrs));

    match (self.evaluate_ifdef(defined, attrs), embedded_line.as_str()) {
      (false, "") => Ok(DirectiveAction::SkipLinesUntilEndIf),
      (false, _) => Ok(DirectiveAction::ReadNextLine),
      (true, "") => Ok(DirectiveAction::ReadNextLine),
      (true, embedded) => {
        let mut src = BumpString::with_capacity_in(embedded.len() + 1, self.bump);
        src.push_str(embedded);
        src.push('\n');
        let offset = line.first_loc().unwrap().start + embedded_line.start() as u32;
        self.lexer.set_tmp_buf(&src, BufLoc::Offset(offset));
        let line = self.read_line()?.unwrap();
        Ok(DirectiveAction::SubstituteLine(line))
      }
    }
  }

  fn evaluate_ifdef(&self, defined: bool, attrs: &str) -> bool {
    let is_any = attrs.contains(',');
    let mut result = attrs.split([',', '+']).fold(!is_any, |current, attr| {
      if !is_any && !current && defined {
        false
      } else if is_any && current {
        true
      } else {
        match self.document.meta.get(attr) {
          None => false,
          Some(AttrValue::String(_)) => true,
          Some(AttrValue::Bool(true)) => true,
          Some(AttrValue::Bool(false)) => false,
        }
      }
    });
    if !defined {
      result = !result;
    }
    result
  }

  pub(crate) fn skip_lines_until_endif(
    &mut self,
    start_line: &Line<'arena>,
  ) -> Result<Option<Line<'arena>>> {
    let mut depth = 1;
    loop {
      let token = self.lexer.next_token();
      if token.matches(TokenKind::Directive, "endif::") {
        let mut line = Line::empty(self.bump);
        line.push(token);
        while !self.lexer.at_newline() && !self.lexer.is_eof() {
          line.push(self.lexer.next_token());
        }
        if line.is_directive_endif() {
          depth -= 1;
          if depth == 0 {
            if self.lexer.at_newline() {
              self.lexer.skip_newline();
            }
            break self.read_line();
          }
        }
      } else if token.kind(TokenKind::Directive)
        && (token.lexeme == "ifdef::" || token.lexeme == "ifndef::")
      {
        // TODO: we should probably check if the directive is valid
        depth += 1;
      } else if self.lexer.is_eof() {
        // TODO: should probably not be a skippable/non-strict error
        self.err_line("This ifdef directive was never closed", start_line)?;
        break Ok(None);
      }
    }
  }
}
