pub mod xref {
  use super::file;
  use ast::{DocumentMeta, ReadAttr, XrefKind};

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
}

pub fn set_backend_attrs<B: crate::Backend>(doc_meta: &mut ast::DocumentMeta) {
  // maybe only if not set?
  doc_meta
    .insert_doc_attr("outfilesuffix", B::OUTFILESUFFIX.to_string())
    .unwrap();
}

pub mod file {
  /// NB: does not return the `.`
  pub fn ext(input: &str) -> Option<&str> {
    if let Some(idx) = input.rfind('.') {
      Some(&input[idx + 1..])
    } else {
      None
    }
  }

  pub fn has_adoc_ext(path: &str) -> bool {
    matches!(
      ext(path),
      Some("adoc") | Some("asciidoc") | Some("asc") | Some("ad")
    )
  }

  pub fn remove_ext(input: &str) -> &str {
    if let Some(idx) = input.rfind('.') {
      &input[..idx]
    } else {
      input
    }
  }

  pub fn basename(input: &str) -> &str {
    input.split(&['/', '\\']).last().unwrap_or(input)
  }

  pub fn stem(input: &str) -> &str {
    basename(input).split('.').next().unwrap_or(input)
  }

  pub fn remove_uri_scheme(input: &str) -> &str {
    let mut split = input.splitn(2, "://");
    let first = split.next().unwrap_or("");
    let Some(rest) = split.next() else {
      return input;
    };
    if rest.is_empty() {
      input
    } else if matches!(first, "http" | "https" | "ftp" | "mailto" | "irc" | "file") {
      rest
    } else {
      input
    }
  }
}
