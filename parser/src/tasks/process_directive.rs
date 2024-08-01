use lazy_static::lazy_static;
use meta::SafeMode;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

struct Directive<'a> {
  line_src: BumpString<'a>,
  first_token: Token<'a>,
  target: BumpString<'a>,
  target_has_spaces: bool,
  target_is_uri: bool,
  attrs: AttrList<'a>,
}

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_directive(
    &mut self,
    line: &mut Line<'arena>,
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
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    let Some(directive) = self.valid_include_directive(line)? else {
      return Ok(DirectiveAction::Passthrough);
    };

    if self.document.meta.safe_mode == SafeMode::Secure {
      // TODO: maybe warn?
      return Ok(DirectiveAction::SubstituteLine(
        self.substitute_link_for_include(&directive),
      ));
    }

    if directive.target_is_uri
      && (self.document.meta.safe_mode > SafeMode::Server
        || !self.document.meta.is_true("allow-uri-read"))
    {
      self.err_at(
        "Cannot include URL contents (allow-uri-read not enabled)",
        directive.first_token.loc.end,
        directive.first_token.loc.end + directive.target.len() as u32,
      )?;
      return Ok(DirectiveAction::SubstituteLine(
        self.substitute_link_for_include(&directive),
      ));
    }

    let Some(resolver) = self.include_resolver.as_mut() else {
      self.err_token_full(
        "No resolver supplied for include directive",
        &directive.first_token,
      )?;
      return Ok(DirectiveAction::Passthrough);
    };

    let mut buffer = BumpVec::new_in(self.bump);
    match resolver.resolve(&directive.target, &mut buffer) {
      Ok(_) => {
        self.lexer.push_source(&directive.target, buffer);
        Ok(DirectiveAction::ReadNextLine)
      }
      Err(error) => {
        self.err_token_full(
          format!("Include resolver returned error: {}", error),
          &directive.first_token,
        )?;
        Ok(DirectiveAction::Passthrough)
      }
    }
  }

  fn valid_include_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<Option<Directive<'arena>>> {
    if line.num_tokens() < 4
      || !line.contains_nonescaped(OpenBracket)
      || !line.ends_with_nonescaped(CloseBracket)
    {
      return Ok(None);
    }
    let src = line.reassemble_src();
    if let Some(captures) = VALID_INCLUDE.captures(&src) {
      let target = captures.get(1).unwrap().as_str();
      let has_spaces = target.contains(' ');
      let target = BumpString::from_str_in(target, self.bump);
      let first_token = line.consume_current().unwrap();
      let target_is_uri = line.current_token().is_url_scheme();
      let num_target_tokens = line.first_nonescaped(TokenKind::OpenBracket).unwrap().1;
      line.discard(num_target_tokens + 1); // including open bracket
      Ok(Some(Directive {
        line_src: src,
        first_token,
        target,
        target_has_spaces: has_spaces,
        target_is_uri,
        attrs: self.parse_attr_list(line)?,
      }))
    } else {
      Ok(None)
    }
  }

  fn substitute_link_for_include(&self, directive: &Directive<'arena>) -> Line<'arena> {
    let link_src = directive.line_src.replace("include::", "link:");
    let mut lexer = Lexer::from_str(self.bump, &link_src);
    lexer.adjust_offset(directive.first_token.loc.start + 4);
    let mut line = lexer.consume_line().unwrap();
    let mut tokens = Line::empty(self.bump);
    let first_token = line.consume_current().unwrap();
    let first_loc = first_token.loc;
    tokens.push(first_token);
    if directive.target_has_spaces {
      let loc = first_loc.clamp_end();
      tokens.push(tok(MacroName, "pass:", loc, self.bump));
      tokens.push_nonpass(tok(Word, "c", loc, self.bump));
      tokens.push_nonpass(tok(OpenBracket, "[", loc, self.bump));
    }
    for token in line.into_iter() {
      if token.kind == OpenBracket {
        let loc = token.loc.clamp_end();
        if directive.target_has_spaces {
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
