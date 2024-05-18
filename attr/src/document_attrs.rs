use std::collections::HashSet;

use crate::internal::*;

#[derive(Debug, Clone, Default)]
pub struct DocumentAttrs {
  authors: Vec<Author>,
  safe_mode: SafeMode,
  doctype: DocType,
  task_attrs: TaskAttrs, // naming?
  doc_attrs: Attrs,
  // remove vvvvv
  finished_header: bool, // this is weird
}

impl DocumentAttrs {
  pub fn new(safe_mode: SafeMode, task_attrs: TaskAttrs) -> Self {
    Self {
      safe_mode,
      doctype: DocType::Article,
      task_attrs,
      doc_attrs: Attrs::default(),
      authors: Vec::new(),
      finished_header: false,
    }
  }

  fn insert_string_attr(&mut self, key: &str, value: String) {
    self.doc_attrs.insert(key, AttrValue::String(value));
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
    self.insert_string_attr(&format!("author_{}", n), author.fullname());
    if let Some(email) = author.email.as_ref() {
      self.insert_string_attr(&format!("email_{}", n), email.clone());
    }
    self.insert_string_attr(&format!("lastname_{}", n), author.last_name.clone());
    if let Some(middle_name) = author.middle_name.as_ref() {
      self.insert_string_attr(&format!("middlename_{}", n), middle_name.clone());
    }
    self.insert_string_attr(&format!("firstname_{}", n), author.first_name.clone());
    self.insert_string_attr(&format!("authorinitials_{}", n), author.initials());

    if let Some(AttrValue::String(authors)) = self.doc_attrs.get("authors") {
      self.insert_string_attr("authors", format!("{}, {}", authors, author.fullname()));
    } else {
      self.insert_string_attr("authors", author.fullname());
    }

    self.authors.push(author);
  }

  pub fn set(&mut self, key: &str, value: AttrValue) -> Result<(), String> {
    if self.finished_header && HEADER_ONLY.contains(key) {
      return Err(format!(
        "Attribute `{}` may only be set in the document header",
        key
      ));
    }
    match key {
      "doctype" => {
        if let Some(doctype) = value.str().and_then(|s| s.parse::<DocType>().ok()) {
          self.doctype = doctype;
          self.doc_attrs.insert(key, value);
        } else {
          return Err("Invalid doctype: expected `article`, `book`, `manpage`, or `inline`".into());
        }
      }
      "chapter-refsig" | "chapter-signifier" | "part-refsig" | "part-signifier"
        if self.doctype != DocType::Book =>
      {
        return Err(format!(
          "Attribute `{}` may only be set when doctype is `book`",
          key
        ));
      }
      _ => self.doc_attrs.insert(key, value),
    }
    Ok(())
  }
}

impl ReadAttr for DocumentAttrs {
  fn get(&self, key: &str) -> Option<&AttrValue> {
    match key {
      // doctype
      "doctype-article" => self.true_if(self.doctype == DocType::Article),
      "doctype-book" => self.true_if(self.doctype == DocType::Book),
      "doctype-inline" => self.true_if(self.doctype == DocType::Inline),
      "doctype-manpage" => self.true_if(self.doctype == DocType::Manpage),

      // safe mode
      "safe-mode-unsafe" => self.true_if(self.safe_mode == SafeMode::Unsafe),
      "safe-mode-safe" => self.true_if(self.safe_mode == SafeMode::Safe),
      "safe-mode-server" => self.true_if(self.safe_mode == SafeMode::Server),
      "safe-mode-secure" => self.true_if(self.safe_mode == SafeMode::Secure),
      "safe-mode-level" => match self.safe_mode {
        SafeMode::Unsafe => Some(&AttrValue::Str("0")),
        SafeMode::Safe => Some(&AttrValue::Str("1")),
        SafeMode::Server => Some(&AttrValue::Str("10")),
        SafeMode::Secure => Some(&AttrValue::Str("20")),
      },
      "safe-mode-name" => match self.safe_mode {
        SafeMode::Unsafe => Some(&AttrValue::Str("UNSAFE")),
        SafeMode::Safe => Some(&AttrValue::Str("SAFE")),
        SafeMode::Server => Some(&AttrValue::Str("SERVER")),
        SafeMode::Secure => Some(&AttrValue::Str("SECURE")),
      },

      // // author
      // "author" if !self.authors.is_empty() => {
      //   let authors = self
      //     .authors
      //     .iter()
      //     .map(|a| a.fullname())
      //     .collect::<Vec<_>>()
      //     .join(", ");
      //   Some(&AttrValue::String(authors))
      // }
      key => match self.task_attrs.get(key) {
        Some(TaskAttr { readonly: true, value }) => Some(value),
        Some(TaskAttr { readonly: false, value }) => self.doc_attrs.get(key).or(Some(value)),
        _ => match self.doc_attrs.get(key) {
          Some(value) => Some(value),
          None => match key {
            // TODO: maybe static? even perfect hash?
            "attribute-missing" => Some(&AttrValue::Str("skip")),
            "attribute-undefined" => Some(&AttrValue::Str("drop-line")),
            "appendix-caption" => Some(&AttrValue::Str("Appendix")),
            "appendix-refsig" => Some(&AttrValue::Str("Appendix")),
            "caution-caption" => Some(&AttrValue::Str("Caution")),
            "chapter-refsig" => Some(&AttrValue::Str("Chapter")),
            "example-caption" => Some(&AttrValue::Str("Example")),
            "figure-caption" => Some(&AttrValue::Str("Figure")),
            "important-caption" => Some(&AttrValue::Str("Important")),
            "last-update-label" => Some(&AttrValue::Str("Last updated")),
            "note-caption" => Some(&AttrValue::Str("Note")),
            "part-refsig" => Some(&AttrValue::Str("Part")),
            "section-refsig" => Some(&AttrValue::Str("Section")),
            "table-caption" => Some(&AttrValue::Str("Table")),
            "tip-caption" => Some(&AttrValue::Str("Tip")),
            "toc-title" => Some(&AttrValue::Str("Table of Contents")),
            "untitled-label" => Some(&AttrValue::Str("Untitled")),
            "version-label" => Some(&AttrValue::Str("Version")),
            "warning-caption" => Some(&AttrValue::Str("Warning")),
            // defaults
            _ => None,
          },
        },
      },
      // jared
    }
  }
}

// this should be moved into a test when encountering a decl
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
    ])
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn attr_merging() {
    let mut attrs = DocumentAttrs::default();
    attrs.task_attrs.insert(
      "task_readonly",
      TaskAttr {
        readonly: true,
        value: AttrValue::Bool(true),
      },
    );
    attrs.task_attrs.insert(
      "task_modifiable",
      TaskAttr {
        readonly: false,
        value: AttrValue::Bool(true),
      },
    );

    attrs
      .doc_attrs
      .insert("task_readonly", AttrValue::Bool(false));
    attrs
      .doc_attrs
      .insert("task_modifiable", AttrValue::Bool(false));
    attrs
      .doc_attrs
      .insert("only_doc_set", AttrValue::Bool(false));

    assert!(attrs.is_true("task_readonly"));
    assert!(attrs.is_false("task_modifiable"));
    assert!(attrs.is_false("only_doc_set"));
  }

  #[test]
  fn defaults() {
    let mut attrs = DocumentAttrs::default();
    attrs.task_attrs.insert(
      "doctype",
      TaskAttr {
        readonly: false,
        value: AttrValue::String("article".into()),
      },
    );
    assert!(attrs.is_true("doctype-article"));
    assert_eq!(attrs.str("attribute-missing").unwrap(), "skip");
  }

  #[test]
  fn safe_mode() {
    let mut attrs = DocumentAttrs::default();
    assert!(attrs.is_true("safe-mode-unsafe"));
    assert!(attrs.get("safe-mode-safe").is_none());
    assert_eq!(attrs.u8("safe-mode-level"), Some(0));
    assert_eq!(attrs.str("safe-mode-name"), Some("UNSAFE"));
    attrs.safe_mode = SafeMode::Server;
    assert!(attrs.is_true("safe-mode-server"));
    assert_eq!(attrs.u8("safe-mode-level"), Some(10));
    assert!(attrs.get("safe-mode-unsafe").is_none());
    assert_eq!(attrs.str("safe-mode-name"), Some("SERVER"));
  }

  #[test]
  fn doctype() {
    let mut attrs = DocumentAttrs::default();
    assert!(attrs.is_true("doctype-article"));
    attrs.set("doctype", AttrValue::Str("book")).unwrap();
    assert!(attrs.get("doctype-article").is_none());
    assert!(attrs.is_true("doctype-book"));
  }

  #[test]
  fn authors() {
    // single author from author line
    let mut attrs = DocumentAttrs::default();
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
