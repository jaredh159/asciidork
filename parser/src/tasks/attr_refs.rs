use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn set_source_file_attrs(&mut self) {
    let source_file = self.lexer.source_file().clone();
    match source_file {
      SourceFile::Stdin { .. } => {}
      SourceFile::Path(path) => {
        self.insert_file_attr("docfilesuffix", path.ext().to_string());
        self.insert_file_attr("docname", path.basename_no_ext().to_string());
        match self.document.meta.safe_mode {
          SafeMode::Server | SafeMode::Secure => {
            self.insert_file_attr("docdir", "");
            self.insert_file_attr("docfile", "");
          }
          SafeMode::Safe | SafeMode::Unsafe => {
            self.insert_file_attr("docfile", path.to_string());
            self.insert_file_attr("docdir", path.parent().to_string());
          }
        }
      }
      SourceFile::Tmp => {}
    };
  }

  pub(crate) fn push_token_replacing_attr_ref(
    &mut self,
    mut token: Token<'arena>,
    line: &mut Line<'arena>,
    drop_line: &mut bool,
  ) -> Result<()> {
    if token.is(TokenKind::AttrRef) && self.ctx.subs.attr_refs() {
      match self.document.meta.get(token.attr_name()) {
        Some(AttrValue::String(attr_val)) => {
          if !attr_val.is_empty() {
            self.lexer.set_tmp_buf(attr_val, BufLoc::Repeat(token.loc));
          }
          line.push(token);
        }
        _ => match self.document.meta.str("attribute-missing") {
          Some("drop") => {}
          Some("drop-line") => *drop_line = true,
          val => {
            token.kind = TokenKind::Word;
            if val == Some("warn") {
              self.err_token_full("Skipping reference to missing attribute", &token)?;
            }
            line.push(token);
          }
        },
      }
    } else {
      line.push(token);
    }
    Ok(())
  }

  fn insert_file_attr(&mut self, key: &str, value: impl Into<AttrValue>) {
    self
      .document
      .meta
      .insert_job_attr(key, JobAttr::readonly(value))
      .unwrap();
  }
}
