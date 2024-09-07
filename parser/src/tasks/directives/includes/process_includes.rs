use lazy_static::lazy_static;
use meta::SafeMode;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug)]
struct IncludeDirective<'a> {
  line_src: BumpString<'a>,
  first_token: Token<'a>,
  target_str: BumpString<'a>,
  target_has_spaces: bool,
  target_is_uri: bool,
  attrs: AttrList<'a>,
}

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_include_directive(
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
      self.target_err(
        "Cannot include URL contents (allow-uri-read not enabled)",
        &directive,
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

    let target = match super::target::prepare(
      directive.target_str.as_str(),
      directive.target_is_uri,
      self.lexer.source_file(),
      self.lexer.source_is_primary(),
      resolver.get_base_dir().map(Path::new),
      self.document.meta.safe_mode,
    ) {
      Ok(target) => target,
      Err(err) => {
        self.target_err(format!("Error preparing target: {}", err), &directive)?;
        return Ok(DirectiveAction::SubstituteLine(
          self.substitute_link_for_include(&directive),
        ));
      }
    };

    let target_abspath = target.path();
    let mut buffer = BumpVec::new_in(self.bump);
    match resolver.resolve(target, &mut buffer) {
      Ok(_) => {
        if let Err(msg) = self.normalize_include_bytes(&target_abspath, &mut buffer) {
          self.target_err(format!("Error resolving file contents: {msg}"), &directive)?;
          return Ok(DirectiveAction::SubstituteLine(
            self.substitute_link_for_include(&directive),
          ));
        }
        self.lexer.push_source(target_abspath, buffer);
        Ok(DirectiveAction::ReadNextLine)
      }
      Err(error) => {
        self.target_err(format!("Include resolver error: {}", error), &directive)?;
        Ok(DirectiveAction::Passthrough)
      }
    }
  }

  fn valid_include_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<Option<IncludeDirective<'arena>>> {
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
      Ok(Some(IncludeDirective {
        line_src: src,
        first_token,
        target_str: target,
        target_has_spaces: has_spaces,
        target_is_uri,
        attrs: self.parse_attr_list(line)?,
      }))
    } else {
      Ok(None)
    }
  }

  fn substitute_link_for_include(&mut self, directive: &IncludeDirective<'arena>) -> Line<'arena> {
    let mut link_src = directive.line_src.replace("include::", "link:");
    link_src.push('\n');
    let offset = directive.first_token.loc.start + 4;
    self.lexer.set_tmp_buf(&link_src, BufLoc::Offset(offset));
    let mut line = self.read_line().unwrap().unwrap();
    let mut tokens = Line::empty(self.bump);
    let first_token = line.consume_current().unwrap();
    let first_loc = first_token.loc;
    tokens.push(first_token);
    if directive.target_has_spaces {
      let loc = first_loc.clamp_end();
      tokens.push(self.token(MacroName, "pass:", loc));
      tokens.push_nonpass(self.token(Word, "c", loc));
      tokens.push_nonpass(self.token(OpenBracket, "[", loc));
    }
    for token in line.into_iter() {
      if token.kind == OpenBracket {
        let loc = token.loc.clamp_end();
        if directive.target_has_spaces {
          tokens.push_nonpass(self.token(CloseBracket, "]", loc));
        }
        tokens.push_nonpass(token);
        tokens.push_nonpass(self.token(Word, "role", loc));
        tokens.push_nonpass(self.token(EqualSigns, "=", loc));
        tokens.push_nonpass(self.token(Word, "include", loc));
        tokens.push_nonpass(self.token(Comma, ",", loc));
      } else {
        tokens.push(token);
      }
    }
    tokens
  }

  fn target_err(
    &mut self,
    msg: impl Into<String>,
    directive: &IncludeDirective<'arena>,
  ) -> Result<()> {
    self.err_at(
      msg,
      directive.first_token.loc.end,
      directive.first_token.loc.end + directive.target_str.len() as u32,
    )
  }
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
