use crate::internal::*;
use crate::variants::token::*;

impl<'bmp> Parser<'bmp> {
  pub(crate) fn try_process_directive(&mut self, line: &Line<'bmp>) -> Result<bool> {
    match line.current_token().unwrap().lexeme.as_str() {
      "include::" => self.try_process_include_directive(line),
      "ifdef::" => todo!(),
      "ifndef::" => todo!(),
      "ifeval::" => todo!(),
      _ => unreachable!("Parser::try_process_directive"),
    }
  }

  // IncludeDirectiveRx = /^(\\)?include::([^\s\[](?:[^\[]*[^\s\[])?)\[(#{CC_ANY}+)?\]$/
  fn try_process_include_directive(&mut self, line: &Line<'bmp>) -> Result<bool> {
    // consider regex instead?
    if line.num_tokens() < 4
      || !line.contains_nonescaped(OpenBracket)
      || !line.ends_with_nonescaped(CloseBracket)
    {
      return Ok(false);
    }
    let mut line = line.clone();
    let _line_start = line.loc().unwrap();
    line.discard_assert(Directive);
    let mut target = BumpString::with_capacity_in(line.src_len(), self.bump);
    let first = line.consume_current().unwrap();
    target.push_str(&first.lexeme);
    let mut last_kind = first.kind;
    loop {
      if line.is_empty() || (line.current_is(OpenBracket) && last_kind != Backslash) {
        break;
      }
      let token = line.consume_current().unwrap();
      target.push_str(&token.lexeme);
      last_kind = token.kind;
    }
    if !line.current_is(OpenBracket) {
      return Ok(false);
    }
    line.discard(1);
    let _attrs = self.parse_attr_list(&mut line)?;
    let Some(resolver) = self.include_resolver.as_mut() else {
      // TODO: is this correct?
      self.err_token_full("No resolver found for include directive", &first)?;
      return Ok(false);
    };

    let mut buffer = BumpVec::new_in(self.bump);
    resolver.resolve(&target, &mut buffer).unwrap();
    self.lexer.push_source(&target, buffer);
    Ok(true)
  }
}

// include::target[
//    leveloffset=offset,
//    lines=ranges,
//    tag(s)=name(s),
//    indent=depth,
//    encoding=encoding,
//    opts=optional
// ]
