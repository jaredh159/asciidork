use crate::internal::*;
use crate::variants::token::*;
use ast::short::block::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn parse_image_block(
    &mut self,
    mut lines: ContiguousLines<'arena>,
    meta: ChunkMeta<'arena>,
  ) -> Result<Block<'arena>> {
    let mut line = lines.consume_current().unwrap();
    let loc = line.loc().unwrap();
    line.discard_assert(MacroName);
    line.discard_assert(Colon);
    let is_uri = line.current_token().kind(UriScheme);
    let target = line.consume_macro_target(self.bump);
    let attrs = self.parse_block_attr_list(&mut line)?;
    self.restore_lines(lines);
    let kind = self.parse_image_kind(&target, is_uri, &attrs, Some(&meta.attrs))?;
    Ok(Block {
      meta,
      context: Context::Image,
      content: Content::Empty(EmptyMetadata::Image { target, attrs, kind }),
      loc: loc.into(),
    })
  }

  pub(crate) fn parse_image_kind(
    &mut self,
    target: &SourceString<'arena>,
    target_is_uri: bool,
    macro_attrs: &AttrList<'arena>,
    block_attrs: Option<&MultiAttrList<'arena>>,
  ) -> Result<ImageKind<'arena>> {
    if self.should_resolve_as_inline_svg(target, macro_attrs, block_attrs) {
      self.parse_inline_svg(target, target_is_uri, macro_attrs)
    } else if self.document.meta.is_true("data-uri")
      && !target.starts_with("data:")
      && (!target_is_uri || self.document.meta.is_true("allow-uri-read"))
    {
      self.parse_data_uri(target, target_is_uri)
    } else {
      Ok(ImageKind::Standard)
    }
  }

  pub(crate) fn maybe_set_admonition_icon_uri(
    &mut self,
    context: BlockContext,
    meta: &ChunkMeta<'arena>,
  ) -> Result<()> {
    let Ok(admonition_kind) = AdmonitionKind::try_from(context) else {
      return Ok(());
    };
    if !self.document.meta.is_true("data-uri") {
      return Ok(());
    }
    if !self.document.meta.is_true("icons") || self.document.meta.str("icons") == Some("font") {
      return Ok(());
    }

    let mut filename = BumpString::with_capacity_in(12, self.bump);
    if let Some(custom) = meta.attrs.named("icon") {
      filename.push_str(custom);
    } else {
      filename.push_str(admonition_kind.lowercase_str());
    }
    filename.push('.');
    let mut ext = if let Some(icon_type) = self.document.meta.str("icontype") {
      icon_type
    } else {
      self.document.meta.str_or("icons", "png")
    };
    ext = iff!(ext.is_empty(), "png", ext);
    filename.push_str(ext);

    let mut data_uri = String::with_capacity(32);
    data_uri.push_str("data:image/");
    data_uri.push_str(iff!(ext == "svg", "svg+xml", ext));
    data_uri.push_str(";base64,");

    if let Some(bytes) = self.get_image_bytes(
      &filename,
      false,
      meta.start_loc,
      IncludeKind::AdmonitionIcon,
    )? {
      data_uri.reserve(bytes.len());
      let encoded = crate::base64::encode_in(&bytes, self.bump);
      data_uri.push_str(&encoded);
    }
    self
      .document
      .meta
      .data_admonition_icons
      .insert(meta.start_loc.uid(), data_uri);
    Ok(())
  }

  fn parse_data_uri(
    &mut self,
    target: &SourceString<'arena>,
    target_is_uri: bool,
  ) -> Result<ImageKind<'arena>> {
    if let Some(bytes) =
      self.get_image_bytes(&target.src, target_is_uri, target.loc, IncludeKind::DataUri)?
    {
      Ok(ImageKind::DataUri(Some(crate::base64::encode_in(
        &bytes, self.bump,
      ))))
    } else {
      Ok(ImageKind::DataUri(None))
    }
  }

  pub(crate) fn parse_inline_svg(
    &mut self,
    target: &SourceString<'arena>,
    target_is_uri: bool,
    attrs: &AttrList<'arena>,
  ) -> Result<ImageKind<'arena>> {
    let Some(buffer) = self.get_image_bytes(
      &target.src,
      target_is_uri,
      target.loc,
      IncludeKind::InlineSvg,
    )?
    else {
      return Ok(ImageKind::InlineSvg(None));
    };
    let mut buf = &buffer[..];
    if let Some(caps) = regx::SVG_PREAMBLE.captures(buf) {
      buf = &buffer[(caps[0].len() - 4)..];
    }
    buf = buf.trim_ascii();

    let width = attrs.named("width").or_else(|| attrs.str_positional_at(1));
    let height = attrs.named("height").or_else(|| attrs.str_positional_at(2));
    let image = if width.is_none() && height.is_none() {
      BumpString::from_utf8_lossy_in(buf, self.bump)
    } else {
      let mut start = BumpString::with_capacity_in(48, self.bump);
      start.push_str("<svg ");
      if let Some(width) = width {
        start.push_str("width=\"");
        start.push_str(width);
        start.push('"');
      }
      if let Some(height) = height {
        start.push_str("height=\"");
        start.push_str(height);
        start.push('"');
      }
      let stripped = regx::SVG_STRIP_ATTRS.replace_all(buf, b"");
      let modified = regx::SVG_START_TAG
        .replace(&stripped, &*start.into_bytes())
        .into_owned();
      BumpString::from_utf8_lossy_in(&modified, self.bump)
    };
    if image.is_empty() {
      self.err_at("Empty svg file", target.loc)?;
      return Ok(ImageKind::InlineSvg(None));
    }
    Ok(ImageKind::InlineSvg(Some(image)))
  }

  fn should_resolve_as_inline_svg(
    &self,
    target: &str,
    macro_attrs: &AttrList<'arena>,
    block_attrs: Option<&MultiAttrList<'arena>>,
  ) -> bool {
    if (!macro_attrs.has_option("inline") && block_attrs.is_none_or(|a| !a.has_option("inline")))
      || macro_attrs.named("format").is_some_and(|f| f != "svg")
    {
      return false;
    }
    if self.document.meta.safe_mode >= SafeMode::Secure {
      return false;
    }
    if macro_attrs.named("format").is_none() && !regx::SVG_TARGET.is_match(target) {
      return false;
    }
    true
  }

  fn get_image_bytes(
    &mut self,
    target: &str,
    target_is_uri: bool,
    err_loc: SourceLocation,
    kind: IncludeKind,
  ) -> Result<Option<BumpVec<'arena, u8>>> {
    let err_str = kind.err_str();
    let Some(resolver) = self.include_resolver.as_mut() else {
      self.err_at(
        format!("No include resolver supplied for {err_str}"),
        err_loc,
      )?;
      return Ok(None);
    };

    if target_is_uri && !self.document.meta.is_true("allow-uri-read") {
      self.err_at(
        "Cannot include URL contents (allow-uri-read not enabled)",
        err_loc,
      )?;
      return Ok(None);
    }

    let include_target = if target_is_uri {
      IncludeTarget::Uri(target.to_string())
    } else if let Some(base_dir) = resolver.get_base_dir().map(Path::new) {
      let mut path = base_dir;
      if let Some(imagesdir) = self.document.meta.str(kind.image_dir_attr()) {
        path = path.join(imagesdir);
      }
      path = path.join(target);
      IncludeTarget::FilePath(path.to_string())
    } else {
      self.err_at(
        format!("Base dir required to resolve relative-path {err_str} for include"),
        err_loc,
      )?;
      return Ok(None);
    };

    let mut buffer = BumpVec::new_in(self.bump);
    match resolver.resolve(include_target, &mut buffer, self.document.meta.safe_mode) {
      Ok(_) => {
        if kind == IncludeKind::InlineSvg
          && let Err(msg) = self.normalize_encoding(None, &mut buffer)
        {
          self.err_at(
            format!("Error resolving file contents for {err_str}: {msg}"),
            err_loc,
          )?;
          return Ok(None);
        }
      }
      Err(msg) => {
        self.err_at(format!("Error including {err_str}: {msg}"), err_loc)?;
        return Ok(None);
      }
    }
    Ok(Some(buffer))
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IncludeKind {
  InlineSvg,
  DataUri,
  AdmonitionIcon,
}

impl IncludeKind {
  const fn err_str(&self) -> &'static str {
    match self {
      IncludeKind::InlineSvg => "inline svg",
      IncludeKind::DataUri => "data uri image",
      IncludeKind::AdmonitionIcon => "data uri admonition icon",
    }
  }

  const fn image_dir_attr(&self) -> &'static str {
    match self {
      IncludeKind::InlineSvg => "imagesdir",
      IncludeKind::DataUri => "imagesdir",
      IncludeKind::AdmonitionIcon => "iconsdir",
    }
  }
}
