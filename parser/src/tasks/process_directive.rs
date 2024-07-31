use lazy_static::lazy_static;
use meta::SafeMode;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_directive(
    &mut self,
    line: &Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    match line.current_token().unwrap().lexeme.as_str() {
      "include::" => self.try_process_include_directive(line),
      "ifdef::" => todo!(),
      "ifndef::" => todo!(),
      "ifeval::" => todo!(),
      "endif::" => todo!(),
      _ => unreachable!("Parser::try_process_directive"),
    }
  }

  fn try_process_include_directive(
    &mut self,
    line: &Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    let Some((src, has_spaces)) = self.valid_include_directive(line) else {
      return Ok(DirectiveAction::Passthrough);
    };
    if self.document.meta.safe_mode == SafeMode::Secure {
      return Ok(DirectiveAction::SubstituteLine(
        self.substitute_link_for_include(src, has_spaces, line.current_token().unwrap().loc.start),
      ));
    }
    let Some((first, target, _attrs)) = self.include_directive_data(line)? else {
      return Ok(DirectiveAction::Passthrough);
    };
    let Some(resolver) = self.include_resolver.as_mut() else {
      // TODO: is this correct?
      self.err_token_full("No resolver found for include directive", &first)?;
      return Ok(DirectiveAction::Passthrough);
    };

    let mut buffer = BumpVec::new_in(self.bump);
    resolver.resolve(&target, &mut buffer).unwrap();
    self.lexer.push_source(&target, buffer);
    Ok(DirectiveAction::ReadNextLine)
  }

  fn valid_include_directive(&self, line: &Line<'arena>) -> Option<(BumpString<'arena>, bool)> {
    if line.num_tokens() < 4
      || !line.contains_nonescaped(OpenBracket)
      || !line.ends_with_nonescaped(CloseBracket)
    {
      return None;
    }
    let src = line.reassemble_src();
    if let Some(captures) = VALID_INCLUDE.captures(&src) {
      let has_spaces = captures.get(1).unwrap().as_str().contains(' ');
      Some((src, has_spaces))
    } else {
      None
    }
  }

  fn include_directive_data(
    &mut self,
    line: &Line<'arena>,
  ) -> Result<Option<(Token<'arena>, BumpString<'arena>, AttrList<'arena>)>> {
    // consider regex instead?
    // IncludeDirectiveRx = /^(\\)?include::([^\s\[](?:[^\[]*[^\s\[])?)\[(#{CC_ANY}+)?\]$/
    if line.num_tokens() < 4
      || !line.contains_nonescaped(OpenBracket)
      || !line.ends_with_nonescaped(CloseBracket)
    {
      return Ok(None);
    }
    let mut line = line.clone();
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
      return Ok(None);
    }
    line.discard(1);
    let attrs = self.parse_attr_list(&mut line)?;
    Ok(Some((first, target, attrs)))
  }

  fn substitute_link_for_include(
    &self,
    line_src: BumpString<'arena>,
    target_has_spaces: bool,
    line_start: u32,
  ) -> Line<'arena> {
    let link_src = line_src.replace("include::", "link:");
    let mut lexer = Lexer::from_str(self.bump, &link_src);
    lexer.adjust_offset(line_start + 4);
    let mut line = lexer.consume_line().unwrap();
    let mut tokens = Line::empty(self.bump);
    let first_token = line.consume_current().unwrap();
    let first_loc = first_token.loc;
    tokens.push(first_token);
    if target_has_spaces {
      let loc = first_loc.clamp_end();
      tokens.push(tok(MacroName, "pass:", loc, self.bump));
      tokens.push_nonpass(tok(Word, "c", loc, self.bump));
      tokens.push_nonpass(tok(OpenBracket, "[", loc, self.bump));
    }
    for token in line.into_iter() {
      if token.kind == OpenBracket {
        let loc = token.loc.clamp_end();
        if target_has_spaces {
          tokens.push_nonpass(tok(CloseBracket, "]", loc, self.bump));
        }
        tokens.push_nonpass(token);
        tokens.push_nonpass(tok(Word, "role", loc, self.bump));
        tokens.push_nonpass(tok(EqualSigns, "=", loc, self.bump));
        tokens.push_nonpass(tok(Word, "include", loc, self.bump));
        tokens.push_nonpass(tok(Comma, ",", loc, self.bump));
      } else {
        tokens.push(token);
      }
    }
    tokens
  }
}

fn tok<'a>(kind: TokenKind, lexeme: &str, loc: SourceLocation, bump: &'a Bump) -> Token<'a> {
  Token::new(kind, loc, BumpString::from_str_in(lexeme, bump))
}

lazy_static! {
  pub static ref VALID_INCLUDE: Regex = Regex::new(r#"^include::([^\[]+[^\[\s])\[.*\]$"#).unwrap();
  // ascidoctor impl: /^(\\)?include::([^\s\[](?:[^\[]*[^\s\[])?)\[(.+)?\]$/
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_includes() {
    assert!(VALID_INCLUDE.is_match("include::valid.adoc[]"));
  }

  #[test]
  fn invalid_includes() {
    assert!(!VALID_INCLUDE.is_match("include::invalid []"));
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
