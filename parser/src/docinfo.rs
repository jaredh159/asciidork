use crate::internal::*;
use std::borrow::Cow;

impl<'arena> Parser<'arena> {
  pub(crate) fn resolve_docinfo(&mut self) {
    if self.document.meta.safe_mode >= SafeMode::Secure {
      return;
    }
    let docinfo = docinfo_attributes(&self.document.meta);
    if docinfo.is_empty() {
      return;
    }
    let Some(mut resolver) = self.include_resolver.take() else {
      return;
    };

    self.document.docinfo.head = self.resolve_docinfo_location(
      resolver.as_mut(),
      &docinfo,
      &docinfo_filename(&self.document.meta, ""),
      "shared-head",
      "private-head",
    );
    self.document.docinfo.header = self.resolve_docinfo_location(
      resolver.as_mut(),
      &docinfo,
      &docinfo_filename(&self.document.meta, "-header"),
      "shared-header",
      "private-header",
    );
    self.document.docinfo.footer = self.resolve_docinfo_location(
      resolver.as_mut(),
      &docinfo,
      &docinfo_filename(&self.document.meta, "-footer"),
      "shared-footer",
      "private-footer",
    );
    self.include_resolver = Some(resolver);
  }

  fn resolve_docinfo_location(
    &mut self,
    resolver: &mut dyn IncludeResolver,
    docinfo: &[String],
    shared_file: &str,
    shared_key: &str,
    private_key: &str,
  ) -> Option<DocInfoFragment<'arena>> {
    let mut content = InlineNodes::new(self.bump);
    let mut added = false;
    if docinfo
      .iter()
      .any(|value| value == "shared" || value == shared_key)
      && let Some(shared) = self.read_docinfo(resolver, shared_file)
    {
      append_docinfo_fragment(&mut content, shared);
      added = true;
    }
    if let Some(docname) = self.document.meta.str("docname")
      && !docname.is_empty()
      && docinfo
        .iter()
        .any(|value| value == "private" || value == private_key)
    {
      let private_file = format!("{docname}-{shared_file}");
      if let Some(private) = self.read_docinfo(resolver, &private_file) {
        if added {
          append_docinfo_text(self.bump, &mut content, "\n");
        }
        append_docinfo_fragment(&mut content, private);
        added = true;
      }
    }
    added.then_some(content)
  }

  fn read_docinfo(
    &mut self,
    resolver: &mut dyn IncludeResolver,
    file: &str,
  ) -> Option<DocInfoFragment<'arena>> {
    let target = docinfo_target(&self.document.meta, resolver, file);
    let mut buffer = BumpVec::new_in(self.bump);
    resolver
      .resolve(target, &mut buffer, self.document.meta.safe_mode)
      .ok()?;
    self.normalize_encoding(None, &mut buffer).ok()?;
    let content = String::from_utf8_lossy(&buffer)
      .replace("\r\n", "\n")
      .replace('\r', "\n");
    Some(self.parse_docinfo_fragment(&content))
  }

  fn parse_docinfo_fragment(&mut self, text: &str) -> DocInfoFragment<'arena> {
    let subs = docinfo_subs(&self.document.meta);
    let mut nodes = InlineNodes::new(self.bump);
    for line in text.split_inclusive('\n') {
      let mut line_nodes = InlineNodes::new(self.bump);
      self.parse_docinfo_line(line, subs, &mut line_nodes);
      append_docinfo_fragment(&mut nodes, line_nodes);
    }
    nodes
  }

  fn parse_docinfo_line(
    &self,
    line_src: &str,
    (attr_refs, replacements): (bool, bool),
    nodes: &mut DocInfoFragment<'arena>,
  ) {
    let line_src = line_src.strip_suffix('\n').unwrap_or(line_src);
    let Some(line) = self.resolve_docinfo_attr_refs(line_src, attr_refs) else {
      return;
    };
    if !replacements {
      append_docinfo_text(self.bump, nodes, line.as_ref());
      return;
    }
    append_docinfo_symbols(self.bump, nodes, line.as_ref());
  }
}

fn docinfo_attributes(meta: &DocumentMeta) -> Vec<String> {
  match meta.str("docinfo") {
    Some(docinfo) if !docinfo.is_empty() => docinfo
      .split(',')
      .map(str::trim)
      .filter(|part| !part.is_empty())
      .map(ToOwned::to_owned)
      .collect(),
    _ if meta.is_set("docinfo2") => vec!["private".into(), "shared".into()],
    _ if meta.is_set("docinfo1") => vec!["shared".into()],
    Some(_) => vec!["private".into()],
    None => Vec::new(),
  }
}

fn docinfo_target(
  meta: &DocumentMeta,
  resolver: &mut dyn IncludeResolver,
  file: &str,
) -> IncludeTarget {
  let path = if let Some(docinfodir) = meta.str("docinfodir") {
    let dir = Path::new(docinfodir);
    if dir.is_absolute() {
      dir.join(file)
    } else if let Some(base_dir) = resolver.get_base_dir() {
      Path::new(base_dir).join(dir).join(file)
    } else {
      dir.join(file)
    }
  } else if let Some(base_dir) = resolver.get_base_dir() {
    Path::new(base_dir).join(file)
  } else {
    Path::new(file)
  };
  IncludeTarget::from(path)
}

fn docinfo_subs(meta: &DocumentMeta) -> (bool, bool) {
  meta.str("docinfosubs").map_or((true, false), |value| {
    let mut attr_refs = false;
    let mut replacements = false;
    for part in value.split(',').map(str::trim) {
      match part {
        "attributes" => attr_refs = true,
        "replacements" => replacements = true,
        _ => {}
      }
    }
    (attr_refs, replacements)
  })
}

fn docinfo_filename(meta: &DocumentMeta, qualifier: &str) -> String {
  let suffix = meta.str("outfilesuffix").unwrap_or(".html");
  format!("docinfo{qualifier}{suffix}")
}

fn append_docinfo_fragment<'arena>(
  target: &mut DocInfoFragment<'arena>,
  fragment: DocInfoFragment<'arena>,
) {
  target.extend(fragment.into_vec());
}

fn append_docinfo_text<'arena>(
  bump: &'arena Bump,
  nodes: &mut DocInfoFragment<'arena>,
  text: &str,
) {
  if text.is_empty() {
    return;
  }
  if let Some(InlineNode { content: Inline::Text(existing), .. }) = nodes.last_mut() {
    existing.push_str(text);
  } else {
    nodes.push(InlineNode::new(
      Inline::Text(BumpString::from_str_in(text, bump)),
      SourceLocation::default(),
    ));
  };
}

fn append_docinfo_symbols<'arena>(
  bump: &'arena Bump,
  nodes: &mut DocInfoFragment<'arena>,
  text: &str,
) {
  let bytes = text.as_bytes();
  let mut start = 0;
  let mut i = 0;
  while i < bytes.len() {
    let symbol = if bytes[i..].starts_with(b"(TM)") {
      Some((SymbolKind::Trademark, 4))
    } else if bytes[i..].starts_with(b"(C)") {
      Some((SymbolKind::Copyright, 3))
    } else if bytes[i..].starts_with(b"(R)") {
      Some((SymbolKind::Registered, 3))
    } else if bytes[i..].starts_with(b"...") {
      Some((SymbolKind::Ellipsis, 3))
    } else if bytes[i..].starts_with(b"---") {
      Some((SymbolKind::TripleDash, 3))
    } else if bytes[i..].starts_with(b"--") {
      Some((SymbolKind::EmDash, 2))
    } else if bytes[i..].starts_with(b"->") {
      Some((SymbolKind::SingleRightArrow, 2))
    } else if bytes[i..].starts_with(b"=>") {
      Some((SymbolKind::DoubleRightArrow, 2))
    } else if bytes[i..].starts_with(b"<-") {
      Some((SymbolKind::SingleLeftArrow, 2))
    } else if bytes[i..].starts_with(b"<=") {
      Some((SymbolKind::DoubleLeftArrow, 2))
    } else {
      None
    };

    if let Some((symbol, len)) = symbol {
      append_docinfo_text(bump, nodes, &text[start..i]);
      nodes.push(InlineNode::new(
        Inline::Symbol(symbol),
        SourceLocation::default(),
      ));
      i += len;
      start = i;
    } else {
      i += 1;
    }
  }
  append_docinfo_text(bump, nodes, &text[start..]);
}

impl Parser<'_> {
  fn resolve_docinfo_attr_refs<'s>(
    &self,
    line_src: &'s str,
    attr_refs: bool,
  ) -> Option<Cow<'s, str>> {
    if !attr_refs {
      return Some(Cow::Borrowed(line_src));
    }

    let attr_missing = self.document.meta.str("attribute-missing");
    let mut drop_line = false;
    let replaced = regx::ATTR_VAL_REPLACE.replace_all(line_src, |caps: &regex::Captures| {
      let attr_name = caps.get(1).unwrap().as_str();
      match self.document.meta.get(attr_name) {
        Some(AttrValue::String(value)) => value.to_string(),
        _ => match attr_missing {
          Some("drop") => String::new(),
          Some("drop-line") => {
            drop_line = true;
            String::new()
          }
          _ => caps.get(0).unwrap().as_str().to_string(),
        },
      }
    });

    (!drop_line).then_some(replaced)
  }
}
