use std::path::Path;
pub mod xref {
  use ast::{DocumentMeta, ReadAttr, XrefKind};
  use core::file;

  pub fn href(target: &str, doc_meta: &DocumentMeta, xref_kind: XrefKind, with_id: bool) -> String {
    if !is_interdoc(target, xref_kind) {
      format!("#{}", remove_leading_hash(target))
    } else {
      interdoc_xref_href(target, doc_meta, xref_kind, with_id)
    }
  }

  fn interdoc_xref_href(
    target: &str,
    doc_meta: &DocumentMeta,
    xref_kind: XrefKind,
    with_id: bool,
  ) -> String {
    let mut parts = target.splitn(2, '#');
    let path = parts.next().unwrap();
    let id = parts.next().filter(|id| !id.is_empty());
    let mut href = String::with_capacity(path.len() + 8 + id.map_or(0, str::len));

    if xref_path_was_included(path, doc_meta) {
      href.push('#');
      if let Some(id) = id {
        href.push_str(id);
      }
      return href;
    }

    push_interdoc_prefix(&mut href, doc_meta);
    if file::has_adoc_ext(path) {
      href.push_str(file::remove_ext(path));
    } else {
      href.push_str(path);
    }
    if xref_kind == XrefKind::Shorthand
      || file::has_adoc_ext(path)
      || !file::basename(path).contains('.')
    {
      push_interdoc_suffix(&mut href, doc_meta);
    }
    if !with_id {
      return href;
    }
    if let Some(id) = id {
      href.push('#');
      href.push_str(id);
    }
    href
  }

  fn xref_path_was_included(path: &str, doc_meta: &DocumentMeta) -> bool {
    if file::has_adoc_ext(path) {
      if Some(path) == doc_meta.str("asciidork-docfilename") {
        true
      } else {
        doc_meta
          .included_files
          .iter()
          .any(|included_file| included_file == path)
      }
    } else if !file::has_ext(path) {
      doc_meta
        .included_files
        .iter()
        .any(|included_file| file::remove_ext(included_file) == path)
    } else {
      false
    }
  }

  fn push_interdoc_suffix(href: &mut String, doc_meta: &DocumentMeta) {
    let suffix = doc_meta
      .str("relfilesuffix")
      .unwrap_or_else(|| doc_meta.str("outfilesuffix").unwrap_or(""));
    href.push_str(suffix);
  }

  fn push_interdoc_prefix(href: &mut String, doc_meta: &DocumentMeta) {
    if let Some(prefix) = doc_meta.str("relfileprefix") {
      href.push_str(prefix);
    }
  }

  pub fn is_interdoc(target: &str, xref_kind: XrefKind) -> bool {
    if target.starts_with('#') {
      return false;
    }
    match xref_kind {
      XrefKind::Shorthand => target.contains('#'),
      XrefKind::Macro => target.contains('#') || file::basename(target).contains('.'),
    }
  }

  pub fn remove_leading_hash(input: &str) -> &str {
    input.strip_prefix('#').unwrap_or(input)
  }

  pub fn get_id(target: &str) -> &str {
    target.split_once('#').map(|x| x.1).unwrap_or(target)
  }
}

pub fn file_ext(path: &str) -> Option<&str> {
  Path::new(path).extension().and_then(|s| s.to_str())
}

pub fn set_backend_attrs<B: crate::Backend>(doc_meta: &mut ast::DocumentMeta) {
  // maybe only if not set?
  doc_meta
    .insert_doc_attr("outfilesuffix", B::OUTFILESUFFIX.to_string())
    .unwrap();
}

#[macro_export]
macro_rules! num_str {
  ($n:expr) => {
    match $n {
      0 => std::borrow::Cow::Borrowed("0"),
      1 => std::borrow::Cow::Borrowed("1"),
      2 => std::borrow::Cow::Borrowed("2"),
      3 => std::borrow::Cow::Borrowed("3"),
      4 => std::borrow::Cow::Borrowed("4"),
      5 => std::borrow::Cow::Borrowed("5"),
      6 => std::borrow::Cow::Borrowed("6"),
      _ => std::borrow::Cow::Owned($n.to_string()),
    }
  };
}
