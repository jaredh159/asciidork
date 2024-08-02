use crate::internal::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct JobSettings {
  pub doctype: Option<DocType>,
  pub safe_mode: SafeMode,
  pub job_attrs: JobAttrs,
  pub embedded: bool, // TODO: not needed by parser, consider making backend-only
  pub strict: bool,   // TODO: expand to log-level and failure-level
}

impl JobSettings {
  pub fn embedded() -> Self {
    Self { embedded: true, ..Default::default() }
  }

  pub fn inline() -> Self {
    Self {
      doctype: Some(DocType::Inline),
      ..Default::default()
    }
  }

  pub fn safe() -> Self {
    Self {
      safe_mode: SafeMode::Safe,
      ..Default::default()
    }
  }

  pub fn r#unsafe() -> Self {
    Self {
      safe_mode: SafeMode::Unsafe,
      ..Default::default()
    }
  }

  pub fn secure() -> Self {
    Self {
      safe_mode: SafeMode::Secure,
      ..Default::default()
    }
  }
}

impl Default for JobSettings {
  fn default() -> Self {
    Self {
      doctype: None,
      safe_mode: SafeMode::default(),
      job_attrs: JobAttrs::default(),
      embedded: false,
      strict: true,
    }
  }
}

impl From<JobSettings> for DocumentMeta {
  fn from(settings: JobSettings) -> Self {
    let JobSettings {
      mut job_attrs, doctype, safe_mode, ..
    } = settings;
    if let Some(doctype) = doctype {
      // if they set the doctype at the job level, setting it as a job_attr
      // ensures it can never be overwritten
      job_attrs.insert_unchecked("doctype", JobAttr::readonly(doctype.to_str()));
    }
    let mut meta = DocumentMeta::new(safe_mode, job_attrs);
    meta.embedded = settings.embedded;
    if let Some(doctype) = doctype {
      meta.set_doctype(doctype);
    }
    meta
  }
}
