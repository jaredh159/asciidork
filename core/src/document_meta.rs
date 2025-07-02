use std::collections::HashSet;

use crate::internal::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentMeta {
  authors: Vec<Author>,
  doctype: DocType,
  job_attrs: JobAttrs,
  doc_attrs: Attrs,
  default_attrs: Attrs,
  header_attrs: Attrs,
  pub safe_mode: SafeMode,
  pub embedded: bool,
  pub included_files: HashSet<String>,
}

impl Default for DocumentMeta {
  fn default() -> Self {
    Self {
      safe_mode: SafeMode::default(),
      doctype: DocType::default(),
      job_attrs: JobAttrs::empty(),
      header_attrs: Attrs::empty(),
      doc_attrs: Attrs::empty(),
      default_attrs: Attrs::defaults(),
      authors: Vec::new(),
      embedded: false,
      included_files: HashSet::new(),
    }
  }
}

impl DocumentMeta {
  pub fn new(safe_mode: SafeMode, mut job_attrs: JobAttrs) -> Self {
    match safe_mode {
      SafeMode::Unsafe => {
        job_attrs.insert_unchecked("safe-mode-unsafe", JobAttr::readonly(true));
        job_attrs.insert_unchecked("safe-mode-level", JobAttr::readonly("0"));
        job_attrs.insert_unchecked("safe-mode-name", JobAttr::readonly("UNSAFE"));
      }
      SafeMode::Safe => {
        job_attrs.insert_unchecked("safe-mode-safe", JobAttr::readonly(true));
        job_attrs.insert_unchecked("safe-mode-level", JobAttr::readonly("1"));
        job_attrs.insert_unchecked("safe-mode-name", JobAttr::readonly("SAFE"));
      }
      SafeMode::Server => {
        job_attrs.insert_unchecked("safe-mode-server", JobAttr::readonly(true));
        job_attrs.insert_unchecked("safe-mode-level", JobAttr::readonly("10"));
        job_attrs.insert_unchecked("safe-mode-name", JobAttr::readonly("SERVER"));
      }
      SafeMode::Secure => {
        job_attrs.insert_unchecked("safe-mode-secure", JobAttr::readonly(true));
        job_attrs.insert_unchecked("safe-mode-level", JobAttr::readonly("20"));
        job_attrs.insert_unchecked("safe-mode-name", JobAttr::readonly("SECURE"));
      }
    }
    Self {
      safe_mode,
      doctype: DocType::Article,
      job_attrs,
      header_attrs: Attrs::empty(),
      doc_attrs: Attrs::empty(),
      default_attrs: Attrs::defaults(),
      authors: Vec::new(),
      embedded: false,
      included_files: HashSet::new(),
    }
  }

  pub fn remove_attr(&mut self, key: &str) {
    self.header_attrs.remove(key);
    self.doc_attrs.remove(key);
    self.job_attrs.remove(key);
  }

  pub fn clone_for_cell(&self) -> Self {
    let mut dm = self.clone();
    dm.set_doctype(DocType::Article);
    // toc in asciidoc cells are disconnected, see:
    // https://github.com/asciidoctor/asciidoctor/issues/4017
    dm.remove_attr("toc");
    dm.remove_attr("toc-placement");
    dm.remove_attr("toc-position");
    // https://github.com/asciidoctor/asciidoctor/blob/main/lib/asciidoctor/document.rb#L268
    dm.remove_attr("showtitle");
    dm.remove_attr("notitle");
    dm
  }

  pub fn authors(&self) -> &[Author] {
    &self.authors
  }

  fn insert_string_attr(&mut self, key: &str, value: String) {
    _ = self.header_attrs.insert(key, AttrValue::String(value));
  }

  pub fn add_author(&mut self, author: Author) {
    if self.authors.is_empty() {
      self.insert_string_attr("author", author.fullname());
      if let Some(email) = author.email.as_ref() {
        self.insert_string_attr("email", email.clone());
      }
      self.insert_string_attr("lastname", author.last_name.clone());
      if let Some(middle_name) = author.middle_name.as_ref() {
        self.insert_string_attr("middlename", middle_name.clone());
      }
      self.insert_string_attr("firstname", author.first_name.clone());
      self.insert_string_attr("authorinitials", author.initials());
    }

    let n = self.authors.len() + 1;
    self.insert_string_attr(&format!("author_{n}"), author.fullname());
    if let Some(email) = author.email.as_ref() {
      self.insert_string_attr(&format!("email_{n}"), email.clone());
    }
    self.insert_string_attr(&format!("lastname_{n}"), author.last_name.clone());
    if let Some(middle_name) = author.middle_name.as_ref() {
      self.insert_string_attr(&format!("middlename_{n}"), middle_name.clone());
    }
    self.insert_string_attr(&format!("firstname_{n}"), author.first_name.clone());
    self.insert_string_attr(&format!("authorinitials_{n}"), author.initials());

    if let Some(AttrValue::String(authors)) = self.header_attrs.get("authors") {
      self.insert_string_attr("authors", format!("{}, {}", authors, author.fullname()));
    } else {
      self.insert_string_attr("authors", author.fullname());
    }

    self.authors.push(author);
  }

  pub fn insert_header_attr(
    &mut self,
    key: &str,
    value: impl Into<AttrValue>,
  ) -> Result<(), String> {
    if JOB_ONLY.contains(key) {
      return Err(format!(
        "Attribute `{key}` may only be set at the job level (CLI/API)"
      ));
    }
    let value: AttrValue = value.into();
    match key {
      "doctype" => {
        if let Some(doctype) = value.str().and_then(|s| s.parse::<DocType>().ok()) {
          self.set_doctype(doctype);
        } else {
          return Err("Invalid doctype: expected `article`, `book`, `manpage`, or `inline`".into());
        }
      }
      "chapter-refsig" | "chapter-signifier" | "part-refsig" | "part-signifier"
        if self.doctype != DocType::Book =>
      {
        return Err(format!(
          "Attribute `{key}` may only be set when doctype is `book`"
        ));
      }
      _ => self.header_attrs.insert(key, value)?,
    }
    Ok(())
  }

  pub fn insert_doc_attr(&mut self, key: &str, value: impl Into<AttrValue>) -> Result<(), String> {
    if JOB_ONLY.contains(key) {
      return Err(format!(
        "Attribute `{key}` may only be set at the job level (CLI/API)"
      ));
    }
    if HEADER_ONLY.contains(key) {
      return Err(format!(
        "Attribute `{key}` may only be set in the document header"
      ));
    }
    self.doc_attrs.insert(key, value.into())
  }

  pub fn insert_job_attr(
    &mut self,
    key: impl Into<String>,
    job_attr: JobAttr,
  ) -> Result<(), String> {
    self.job_attrs.insert(key.into(), job_attr)
  }

  pub fn clear_doc_attrs(&mut self) {
    self.doc_attrs = Attrs::empty();
  }

  pub fn set_doctype(&mut self, doctype: DocType) {
    self.doctype = doctype;
    self
      .header_attrs
      .insert("doctype", self.doctype.to_str().into())
      .unwrap();
  }

  pub const fn get_doctype(&self) -> DocType {
    self.doctype
  }

  pub fn icon_mode(&self) -> IconMode {
    match self.get("icons") {
      Some(AttrValue::String(icon)) => match icon.as_str() {
        "font" => IconMode::Font,
        "image" | "" => IconMode::Image,
        _ => IconMode::Text,
      },
      Some(AttrValue::Bool(true)) => IconMode::Image,
      _ => IconMode::Text,
    }
  }

  fn resolve_attr(&self, key: &str) -> Option<&AttrValue> {
    match self.doc_attrs.get(key) {
      Some(value) => Some(value),
      None => match self.header_attrs.get(key) {
        Some(value) => Some(value),
        None => self.default_attrs.get(key),
      },
    }
  }

  pub const fn header_attrs(&self) -> &Attrs {
    &self.header_attrs
  }

  pub fn show_doc_title(&self) -> bool {
    !(self.is_true("notitle")
      || self.is_false("showtitle")
      || (self.embedded && (!self.is_true("showtitle") && !self.is_false("notitle"))))
  }
}

impl ReadAttr for DocumentMeta {
  fn get(&self, key: &str) -> Option<&AttrValue> {
    match key {
      "doctype-article" => self.true_if(self.doctype == DocType::Article),
      "doctype-book" => self.true_if(self.doctype == DocType::Book),
      "doctype-inline" => self.true_if(self.doctype == DocType::Inline),
      "doctype-manpage" => self.true_if(self.doctype == DocType::Manpage),

      key => match self.job_attrs.get(key) {
        Some(JobAttr { readonly: true, value }) => Some(value),
        Some(JobAttr { readonly: false, value }) => self.resolve_attr(key).or(Some(value)),
        _ if key == "doctitle" => self
          .resolve_attr("doctitle")
          .or_else(|| self.resolve_attr("_asciidork_derived_doctitle")),
        _ => self.resolve_attr(key),
      },
    }
  }
}

lazy_static::lazy_static! {
  static ref JOB_ONLY: HashSet<&'static str> = {
    HashSet::from_iter(vec![
      "allow-uri-read",
      "max-attribute-value-size",
      "max-include-depth",
      "doc",
      "docdir",
      "docfile",
      "docfilesuffix",
      "docname",
    ])
  };
}

lazy_static::lazy_static! {
  static ref HEADER_ONLY: HashSet<&'static str> = {
    HashSet::from_iter(vec![
      "experimental",
      "reproducible",
      "skip-front-matter",
      "lang",
      "last-update-label",
      "manname-title",
      "nolang",
      "toc-title",
      "untitled-label",
      "version-label",
      "app-name",
      "author",
      "authorinitials",
      "authors",
      "copyright",
      "doctitle",
      "doctype",
      "description",
      "email",
      "firstname",
      "keywords",
      "lastname",
      "middlename",
      "orgname",
      "revdate",
      "revnumber",
      "revremark",
      "title",
      "copycss",
      "css-signature",
      "linkcss",
      "stylesdir",
      "stylesheet",
      "toc-class",
      "webfonts",
    ])
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn attr_merging() {
    let mut job_attrs = JobAttrs::default();
    job_attrs.insert_unchecked("job_readonly", JobAttr::readonly(true));
    job_attrs.insert_unchecked("job_modifiable", JobAttr::modifiable(true));
    let mut attrs = DocumentMeta::new(SafeMode::Secure, job_attrs);

    assert!(attrs.is_true("job_readonly"));
    assert!(attrs.is_true("job_modifiable"));

    attrs.insert_header_attr("job_readonly", false).unwrap();
    attrs.insert_header_attr("job_modifiable", false).unwrap();
    attrs.insert_header_attr("only_doc_set", false).unwrap();

    assert!(attrs.is_true("job_readonly"));
    assert!(attrs.is_false("job_modifiable"));
    assert!(attrs.is_false("only_doc_set"));

    // doc attrs trump header_attrs
    attrs.insert_header_attr("sectids", true).unwrap();
    attrs.insert_doc_attr("sectids", false).unwrap();
    assert!(attrs.is_false("sectids"));

    // attempting to set read-only job attr has no effect
    attrs.insert_doc_attr("safe-mode-name", "UNSAFE").unwrap();
    assert_eq!(attrs.str("safe-mode-name"), Some("SECURE"));
  }

  #[test]
  fn defaults() {
    let mut attrs = DocumentMeta::default();
    attrs
      .job_attrs
      .insert_unchecked("doctype", JobAttr::readonly("article"));
    assert!(attrs.is_true("doctype-article"));
    assert_eq!(attrs.str("attribute-missing").unwrap(), "skip");
  }

  #[test]
  fn safe_mode() {
    let attrs = DocumentMeta::new(SafeMode::Unsafe, JobAttrs::default());
    assert!(attrs.is_true("safe-mode-unsafe"));
    assert!(attrs.get("safe-mode-safe").is_none());
    assert_eq!(attrs.u8("safe-mode-level"), Some(0));
    assert_eq!(attrs.str("safe-mode-name"), Some("UNSAFE"));
    let attrs = DocumentMeta::new(SafeMode::Server, JobAttrs::default());
    assert!(attrs.is_true("safe-mode-server"));
    assert_eq!(attrs.u8("safe-mode-level"), Some(10));
    assert!(attrs.get("safe-mode-unsafe").is_none());
    assert_eq!(attrs.str("safe-mode-name"), Some("SERVER"));
  }

  #[test]
  fn doctype() {
    let mut attrs = DocumentMeta::default();
    assert!(attrs.is_true("doctype-article"));
    attrs
      .insert_header_attr("doctype", AttrValue::String("book".into()))
      .unwrap();
    assert!(attrs.get("doctype-article").is_none());
    assert!(attrs.is_true("doctype-book"));
  }

  #[test]
  fn authors() {
    // single author from author line
    let mut attrs = DocumentMeta::default();
    attrs.add_author(Author {
      first_name: "John".into(),
      middle_name: Some("M".into()),
      last_name: "Doe".into(),
      email: Some("john@doe.com".into()),
    });
    assert_eq!(attrs.str("author"), Some("John M Doe"));
    assert_eq!(attrs.str("email"), Some("john@doe.com"));
    assert_eq!(attrs.str("firstname"), Some("John"));
    assert_eq!(attrs.str("middlename"), Some("M"));
    assert_eq!(attrs.str("lastname"), Some("Doe"));
    assert_eq!(attrs.str("authorinitials"), Some("JMD"));

    assert_eq!(attrs.str("author_1"), Some("John M Doe"));
    assert_eq!(attrs.str("email_1"), Some("john@doe.com"));
    assert_eq!(attrs.str("firstname_1"), Some("John"));
    assert_eq!(attrs.str("middlename_1"), Some("M"));
    assert_eq!(attrs.str("lastname_1"), Some("Doe"));
    assert_eq!(attrs.str("authorinitials_1"), Some("JMD"));

    assert_eq!(attrs.str("authors"), Some("John M Doe"));

    // with two authors from author line
    attrs.add_author(Author {
      first_name: "Bob".into(),
      middle_name: None,
      last_name: "Smith".into(),
      email: None,
    });
    assert_eq!(attrs.str("author"), Some("John M Doe"));
    assert_eq!(attrs.str("authors"), Some("John M Doe, Bob Smith"));
    assert_eq!(attrs.str("firstname"), Some("John"));
    assert_eq!(attrs.str("author_2"), Some("Bob Smith"));
    assert_eq!(attrs.str("authorinitials_2"), Some("BS"));
    assert_eq!(attrs.get("email_2"), None);
  }
}
