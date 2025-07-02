use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug)]
struct IncludeDirective<'a> {
  line_src: BumpString<'a>,
  first_token: Token<'a>,
  target: BumpString<'a>,
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

    if let Some(max_depth) = self.exceeded_max_include_depth() {
      self.err_line_starting(
        format!("Maximum include depth of {max_depth} exceeded"),
        directive.first_token.loc,
      )?;
      return Ok(DirectiveAction::Passthrough);
    }

    let Some(resolver) = self.include_resolver.as_mut() else {
      self.err_token_full(
        "No resolver supplied for include directive",
        &directive.first_token,
      )?;
      return Ok(DirectiveAction::Passthrough);
    };

    let target = match super::target::prepare(
      directive.target.as_str(),
      directive.target_is_uri,
      self.lexer.source_file(),
      self.lexer.source_is_primary(),
      resolver.get_base_dir().map(Path::new),
    ) {
      Ok(target) => target,
      Err(err) => {
        self.target_err(format!("Error preparing target: {err}"), &directive)?;
        return Ok(DirectiveAction::SubstituteLine(
          self.substitute_link_for_include(&directive),
        ));
      }
    };

    let target_abspath = target.path();
    let mut buffer = BumpVec::new_in(self.bump);
    match resolver.resolve(target, &mut buffer) {
      Ok(_) => {
        if let Err(msg) =
          self.normalize_include_bytes(&target_abspath, &directive.attrs, &mut buffer)
        {
          self.target_err(format!("Error resolving file contents: {msg}"), &directive)?;
          return Ok(DirectiveAction::SubstituteLine(
            self.substitute_link_for_include(&directive),
          ));
        }
        self.select_lines(&directive.attrs, &target_abspath, &mut buffer)?;
        self.set_include_indentation(&directive.attrs, &mut buffer);
        let mut leveloffset = 0;
        if let Some(offset_attr) = directive
          .attrs
          .named("leveloffset")
          .map(|s| AttrValue::String(s.to_string()))
        {
          Parser::adjust_leveloffset(&mut leveloffset, &offset_attr);
        }
        let include_depth = directive
          .attrs
          .named("depth")
          .and_then(|s| s.parse::<u16>().ok());
        self.lexer.push_source(
          SourceFile::Path(target_abspath),
          leveloffset,
          include_depth,
          buffer,
        );
        self
          .document
          .meta
          .included_files
          .insert(directive.target.as_str().to_string());
        Ok(DirectiveAction::ReadNextLine)
      }
      Err(ResolveError::NotFound) if directive.attrs.has_option("optional") => {
        // TODO: when we have info/trace logging, emit a log
        Ok(DirectiveAction::ReadNextLine)
      }
      Err(err @ ResolveError::NotFound | err @ ResolveError::Io(..)) => {
        self.target_err(format!("Include error: {err}"), &directive)?;
        let mut msg = self.string("+++Unresolved directive in ");
        msg.push_str(self.lexer.source_file().file_name());
        msg.push_str(" - ");
        let offset = directive.first_token.loc.start + msg.len() as u32;
        msg.push_str(directive.line_src.as_str());
        msg.push_str("+++");
        self.lexer.set_tmp_buf(&msg, BufLoc::Offset(offset));
        Ok(DirectiveAction::ReadNextLine)
      }
      Err(error) => {
        self.target_err(format!("Include error: {error}"), &directive)?;
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
      let target = self.string(target);
      let first_token = line.consume_current().unwrap();
      let target_is_uri = line.current_token().kind(TokenKind::UriScheme);
      let num_target_tokens = line.first_nonescaped(TokenKind::OpenBracket).unwrap().1;
      line.discard(num_target_tokens + 1); // including open bracket
      Ok(Some(IncludeDirective {
        line_src: src,
        first_token,
        target,
        target_has_spaces: has_spaces,
        target_is_uri,
        attrs: self.parse_block_attr_list(line)?,
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
      SourceLocation::new(
        directive.first_token.loc.end,
        directive.first_token.loc.end + directive.target.len() as u32,
        directive.first_token.loc.include_depth,
      ),
    )
  }

  fn set_include_indentation(&self, attrs: &AttrList, buf: &mut BumpVec<'arena, u8>) {
    if let Some(indent) = attrs.named("indent").and_then(|s| s.parse::<usize>().ok()) {
      _set_indentation(indent, buf, self.bump);
    };
  }

  fn exceeded_max_include_depth(&self) -> Option<u16> {
    let (rel_depth, stack_depth) = self
      .lexer
      .max_include_depth()
      .unwrap_or((self.ctx.max_include_depth, 0));
    let abs_depth = u16::min(rel_depth + stack_depth, self.ctx.max_include_depth);
    if self.lexer.include_depth() + 1 > abs_depth {
      Some(rel_depth)
    } else {
      None
    }
  }
}

fn _set_indentation<'arena>(indent: usize, buf: &mut BumpVec<'arena, u8>, bump: &'arena Bump) {
  let mut trimmed = buf.as_slice();
  if trimmed.last() == Some(&b'\n') {
    trimmed = &trimmed[..trimmed.len() - 1];
  }
  let Some(min_indent) = trimmed
    .split(|&c| c == b'\n')
    .fold(Option::<usize>::None, |acc, line| {
      if line == b"asciidorkinclude::[false]" {
        return acc;
      }
      let line_indent = line.iter().take_while(|&&c| c == b' ').count();
      match acc {
        Some(current) => Some(current.min(line_indent)),
        None => Some(line_indent),
      }
    })
  else {
    return;
  };
  if indent != min_indent {
    let mut dest = BumpVec::with_capacity_in(buf.len(), bump);
    buf.split(|&c| c == b'\n').for_each(|line| {
      let line_indent = line.iter().take_while(|&&c| c == b' ').count();
      if line_indent >= min_indent {
        dest.extend(std::iter::repeat_n(b' ', line_indent - min_indent + indent));
        dest.extend(line.iter().skip(line_indent));
        dest.push(b'\n');
      }
    });
    dest.pop();
    std::mem::swap(buf, &mut dest);
  }
}

lazy_static! {
  pub static ref VALID_INCLUDE: Regex = Regex::new(r#"^include::([^\[]+[^\[\s])\[.*\]$"#).unwrap();
  // ascidoctor impl: /^(\\)?include::([^\s\[](?:[^\[]*[^\s\[])?)\[(.+)?\]$/
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_indent_to_0() {
    let input = "  foo\n    bar\n  baz";
    let mut buf = BumpVec::from_iter_in(input.bytes(), leaked_bump());
    _set_indentation(0, &mut buf, leaked_bump());
    assert_eq!(std::str::from_utf8(&buf).unwrap(), "foo\n  bar\nbaz");
  }

  #[test]
  fn test_indent_to_2() {
    let input = "foo\n  bar\nbaz";
    let mut buf = BumpVec::from_iter_in(input.bytes(), leaked_bump());
    _set_indentation(2, &mut buf, leaked_bump());
    assert_eq!(std::str::from_utf8(&buf).unwrap(), "  foo\n    bar\n  baz");
  }

  #[test]
  fn valid_includes() {
    assert!(VALID_INCLUDE.is_match("include::valid.adoc[]"));
  }

  #[test]
  fn invalid_includes() {
    assert!(!VALID_INCLUDE.is_match("include::invalid []"));
  }
}
