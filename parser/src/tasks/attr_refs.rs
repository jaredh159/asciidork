use std::any::Any;

use crate::internal::*;

pub trait AttrRefObserver: Any {
  fn attr_defined(&mut self, name: &str, value: &AttrValue, name_loc: SourceLocation);
  fn attr_ref_replaced(&mut self, attr_name: &str, loc: SourceLocation);
  fn attr_ref_missing(&mut self, attr_name: &str, loc: SourceLocation);
}

impl<'arena> Parser<'arena> {
  pub(crate) fn set_source_file_attrs(&mut self) {
    let source_file = self.lexer.source_file().clone();
    match source_file {
      SourceFile::Stdin { .. } => {}
      SourceFile::Path(path) => {
        let file_stem = path.file_stem();
        let ext = path.extension();
        self.insert_job_attr("docfilesuffix", ext.to_string());
        self.insert_job_attr("docname", file_stem.to_string());
        self.insert_job_attr("asciidork-docfilename", format!("{}{}", file_stem, ext));
        match self.document.meta.safe_mode {
          SafeMode::Server | SafeMode::Secure => {
            self.insert_job_attr("docdir", "");
            self.insert_job_attr("docfile", "");
          }
          SafeMode::Safe | SafeMode::Unsafe => {
            self.insert_job_attr("docfile", path.to_string());
            self.insert_job_attr("docdir", path.dirname().to_string());
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
    if token.kind(TokenKind::AttrRef) && self.ctx.subs.attr_refs() {
      match self.document.meta.get(&token.lowercase_attr_name()) {
        Some(AttrValue::String(attr_val)) => {
          #[cfg(feature = "attr_ref_observation")]
          if let Some(ref mut observer) = self.attr_ref_observer.as_mut() {
            observer.attr_ref_replaced(token.attr_name(), token.loc);
          }
          if !attr_val.is_empty() {
            self.lexer.set_tmp_buf(attr_val, BufLoc::Repeat(token.loc));
          }
          line.push(token);
        }
        _ => {
          #[cfg(feature = "attr_ref_observation")]
          if let Some(ref mut observer) = self.attr_ref_observer.as_mut() {
            observer.attr_ref_missing(token.attr_name(), token.loc);
          }
          match self.document.meta.str("attribute-missing") {
            Some("drop") => {}
            Some("drop-line") => *drop_line = true,
            val => {
              token.kind = TokenKind::Word;
              if val == Some("warn") {
                self.err_token_full("Skipping reference to missing attribute", &token)?;
              }
              line.push(token);
            }
          }
        }
      }
    } else {
      line.push(token);
    }
    Ok(())
  }

  pub(crate) fn insert_job_attr(&mut self, key: &str, value: impl Into<AttrValue>) {
    self
      .document
      .meta
      .insert_job_attr(key, JobAttr::readonly(value))
      .unwrap();
  }

  pub(crate) fn insert_doc_attr(
    &mut self,
    key: &str,
    value: impl Into<AttrValue>,
  ) -> std::result::Result<(), String> {
    self.document.meta.insert_doc_attr(key, value)
  }
}
