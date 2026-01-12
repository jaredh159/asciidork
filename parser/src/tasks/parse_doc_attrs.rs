use std::borrow::Cow;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_parse_attr_def(&mut self, token: &mut Token<'arena>) -> Result<()> {
    let start_loc = token.loc.start;
    let depth = token.loc.include_depth;

    // we need at least 4 bytes to have a valid attr def: `:a:\n`
    if self.lexer.byte_at(start_loc + 3, depth).is_none() {
      return Ok(());
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum State {
      InName,
      TrailingBang,
      AfterName,
    }
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Negation {
      None,
      Leading,
      Trailing,
    }

    // test attr def start, and bail early
    let mut pos = start_loc + 1;
    let mut negated = Negation::None;
    if self.lexer.byte_at(pos, depth) == Some(b'!') {
      negated = Negation::Leading;
      pos += 1;
    };
    let first_char = self.lexer.byte_at(pos, depth).unwrap();
    if !first_char.is_ascii_alphanumeric() && first_char != b'_' {
      return Ok(());
    }

    let name_start = pos;
    let mut name_end = pos;
    pos += 1;

    let mut state = State::InName;
    let mut has_lbrace = false;
    let mut def_start = u32::MAX;
    loop {
      match (state, self.lexer.byte_at(pos, depth)) {
        (State::InName, Some(b)) if is_attr_word_char(b) => {}
        (State::InName, Some(b'!')) => {
          negated = Negation::Trailing;
          state = State::TrailingBang;
          name_end = pos;
        }
        (State::InName, Some(b':')) => {
          state = State::AfterName;
          name_end = pos;
        }
        (State::InName, _) => {
          return Ok(());
        }
        (State::TrailingBang, Some(b':')) => {
          state = State::AfterName;
          name_end = pos - 1;
        }
        (State::TrailingBang, _) => {
          return Ok(());
        }
        (State::AfterName, Some(b'\r')) if self.lexer.byte_at(pos + 1, depth) == Some(b'\n') => {
          break;
        }
        (State::AfterName, Some(b'\n') | None) => break,
        (State::AfterName, Some(b'{')) => has_lbrace = true,
        (State::AfterName, Some(b)) if def_start == u32::MAX && !b.is_ascii_whitespace() => {
          def_start = pos
        }
        (State::AfterName, _) => {}
      }
      pos += 1;
    }

    if def_start == u32::MAX {
      def_start = name_end;
    }

    let mut attr_def_string = self
      .lexer
      .src_string_from_loc(SourceLocation::new(start_loc, pos, depth))
      .src;
    let name_src = self
      .lexer
      .src_string_from_loc(SourceLocation::new(name_start, name_end, depth));
    let name = &name_src.to_lowercase();

    // gather line continuations
    loop {
      if attr_def_string.ends_with(" \\") {
        attr_def_string.pop();

        // https://docs.asciidoctor.org/asciidoc/latest/attributes/wrap-values/#hard
        if attr_def_string.ends_with(" + ") {
          attr_def_string.pop();
          attr_def_string.push('\n');
        }

        if self.lexer.byte_at(pos + 1, depth) == Some(b'\r') {
          pos += 2;
        } else {
          pos += 1;
        }

        let line_start = pos;
        loop {
          match self.lexer.byte_at(pos, depth) {
            Some(b'\r') if self.lexer.byte_at(pos + 1, depth) == Some(b'\n') => {
              break;
            }
            Some(b'\n') | None => break,
            Some(b'{') => has_lbrace = true,
            Some(_) => {}
          }
          pos += 1;
        }
        let next_line = self
          .lexer
          .str_from_loc(SourceLocation::new(line_start, pos, depth));
        attr_def_string.push_str(next_line);
      } else {
        break;
      }
    }

    let mut val_start = (name_end - start_loc) + 1;
    if negated == Negation::Trailing {
      val_start += 1;
    }

    let value_str = attr_def_string.as_str()[(val_start as usize)..].trim();
    let value = if value_str.is_empty() {
      AttrValue::Bool(negated == Negation::None)
    } else {
      if negated != Negation::None {
        self.err_line_starting("Cannot unset attr with `!` AND provide value", token.loc)?;
      }
      let mut value_string = if has_lbrace && self.ctx.in_header {
        self.replace_attr_vals(value_str).into_owned()
      } else {
        value_str.to_string()
      };
      self.mark_pass_footnote_dbl_attr(&mut value_string);
      AttrValue::String(value_string)
    };

    #[cfg(feature = "attr_ref_observation")]
    if let Some(ref mut observer) = self.attr_ref_observer.as_mut() {
      observer.attr_defined(&name, &value, name_src.loc);
    }

    let attr_def_loc = SourceLocation::new(start_loc, pos, depth);
    self.ctx.attr_defs.push(AttrDef {
      name: name.clone(),
      loc: attr_def_loc,
      val_loc: SourceLocation::new(def_start, pos, depth),
      value: value.clone(),
      has_lbrace,
      in_header: self.ctx.in_header,
    });

    if name == "leveloffset" {
      Parser::adjust_leveloffset(&mut self.ctx.leveloffset, &value);
    }

    if self.ctx.in_header {
      // https://docs.asciidoctor.org/asciidoc/latest/document/author-attribute-entries/
      let mut attr_author: Option<Author> = None;
      if name == "author"
        && self.document.meta.authors.is_empty()
        && let Some(author) = author_from_attr(&value)
      {
        attr_author = Some(author);
      } else if name == "email"
        && self.document.meta.authors.len() == 1
        && let Some(author) = self.document.meta.authors.get_mut(0)
      {
        author.email = value.str().map(|s| s.to_string());
      }
      if let Err(err) = self.document.meta.insert_header_attr(name, value) {
        self.err_at(err, attr_def_loc)?;
      }
      if let Some(author) = attr_author {
        self.document.meta.add_author(author);
      }
    }

    // the token now represents the entire attr def
    token.kind = TokenKind::AttrDef;
    token.loc = attr_def_loc;
    token.lexeme = self.string(self.lexer.str_from_loc(attr_def_loc));
    self.lexer.set_pos(pos);

    Ok(())
  }

  // mark a special case with a sentinal byte, so the lexer can easily recognize it.
  // this helps us work around a place where the implementation and semantics
  // of asciidoctor are basically fundamentally incompatible with our approach
  // of parse -> AST -> eval. asciidoctor documents externalizing footnotes
  // with ininline substitutions like this:
  // ```adoc
  // :fn-disclaimer2: pass:c,q[footnote:disclaimer2[Opinions are _mine_, and mine *alone*.]]
  // A bold statement!{fn-disclaimer2}
  // ```
  // but this implies transforming the text inside the footnote attr into html first and
  // holding it as the attr ref, so the footnote can be dropped into context and expanded.
  // asciidoctor special-cases pass-macros surrounding attr defs to support this, so we
  // are doing something similar, though for now only supporting nested footnote macros, as
  // they are the most likely to appear in this trick, as most other macros a) aren't put in
  // attr refs, and b) don't tend to have inline formatting within the attr list brackets
  fn mark_pass_footnote_dbl_attr(&self, value: &mut String) {
    if !value.starts_with("pass:") || !regx::PASS_DBL_MACRO_ATTR.is_match(value) {
      return;
    }
    // SAFETY: we just checked that the string starts with `pass:`
    // so we're fine to overwrite the first ascii byte with another
    unsafe {
      value.as_mut_vec()[0] = b'\x1b';
    }
  }

  pub(crate) fn replace_attr_vals<'h>(&self, haystack: &'h str) -> Cow<'h, str> {
    regx::ATTR_VAL_REPLACE.replace_all(haystack, |caps: &regex::Captures| {
      if let Some(AttrValue::String(replace)) =
        self.document.meta.get(caps.get(1).unwrap().as_str())
      {
        replace
      } else {
        ""
      }
    })
  }
}

// https://docs.asciidoctor.org/asciidoc/latest/attributes/names-and-values/#user-defined
const fn is_attr_word_char(c: u8) -> bool {
  c.is_ascii_alphanumeric() || c == b'_' || c == b'-'
}

fn author_from_attr(attr: &AttrValue) -> Option<Author> {
  let author_str = attr.str().filter(|s| !s.is_empty())?;
  let parts: Vec<&str> = author_str.split_whitespace().collect();
  Some(Author {
    first_name: parts.first().unwrap_or(&"").replace('_', " ").to_string(),
    middle_name: if parts.len() > 2 {
      Some(parts[1..parts.len() - 1].join(" ").replace('_', " "))
    } else {
      None
    },
    last_name: parts
      .get(parts.len().saturating_sub(1))
      .unwrap_or(&"")
      .replace('_', " ")
      .to_string(),
    email: None,
  })
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_leveloffset() {
    let cases = vec![
      (0, AttrValue::String("1".into()), 1),
      (0, AttrValue::String("+1".into()), 1),
      (0, AttrValue::String("+4".into()), 4),
      (1, AttrValue::String("+1".into()), 2),
      (2, AttrValue::String("-1".into()), 1),
      (2, AttrValue::String("-6".into()), -4),
      (0, AttrValue::Bool(false), 0),
      (1, AttrValue::Bool(false), 0),
      (4, AttrValue::Bool(false), 0),
    ];
    for (mut initial, attr_value, expected) in cases {
      Parser::adjust_leveloffset(&mut initial, &attr_value);
      assert_eq!(initial, expected);
    }
  }

  assert_error!(
    test_parse_doc_attr_error_str,
    adoc! {"
      :doctype: bad

      para
    "},
    error! {"
       --> test.adoc:1:1
        |
      1 | :doctype: bad
        | ^^^^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  assert_error!(
    test_parse_doc_attr_error_unset,
    adoc! {"
      :!doctype:

      para
    "},
    error! {"
       --> test.adoc:1:1
        |
      1 | :!doctype:
        | ^^^^^^^^^^ Invalid doctype: expected `article`, `book`, `manpage`, or `inline`
    "}
  );

  assert_error!(
    doc_attr_error_invalid,
    adoc! {"
      :doctype: article
      :chapter-refsig: Capitulo

      para
    "},
    error! {"
       --> test.adoc:2:1
        |
      2 | :chapter-refsig: Capitulo
        | ^^^^^^^^^^^^^^^^^^^^^^^^^ Attribute `chapter-refsig` may only be set when doctype is `book`
    "}
  );
}
