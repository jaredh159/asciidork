use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn set_source_file_attrs(&mut self) {
    let source_file = self.lexer.source_file().clone();
    match self.document.meta.safe_mode {
      SafeMode::Server | SafeMode::Secure => self.insert_file_attr("docdir", ""),
      SafeMode::Safe | SafeMode::Unsafe => match source_file {
        SourceFile::Stdin { .. } => {}
        SourceFile::Path(path) => self.insert_file_attr("docdir", path.parent().to_string()),
        SourceFile::Tmp => {}
      },
    }
  }

  fn insert_file_attr(&mut self, key: &str, value: impl Into<AttrValue>) {
    self
      .document
      .meta
      .insert_job_attr(key, JobAttr::readonly(value))
      .unwrap();
  }
}
