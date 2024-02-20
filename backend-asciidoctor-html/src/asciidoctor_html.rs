use crate::internal::*;

#[derive(Debug, Default)]
pub struct AsciidoctorHtml {
  html: String,
  alt_html: String,
  footnotes: Vec<(String, String)>,
  doc_attrs: AttrEntries,
  fig_caption_num: usize,
  flags: Flags,
  list_stack: Vec<bool>,
}

impl Backend for AsciidoctorHtml {
  type Output = String;
  type Error = Infallible;

  fn enter_document(&mut self, document: &Document, attrs: &AttrEntries, flags: Flags) {
    self.flags = flags;
    self.doc_attrs = attrs.clone();
    if flags.embedded {
      return;
    }
    self.push_str(r#"<!DOCTYPE html><html"#);
    if !attrs.is_set("nolang") {
      self.push([r#" lang=""#, attrs.str_or("lang", "en"), "\""]);
    }
    let encoding = attrs.str_or("encoding", "UTF-8");
    self.push([r#"><head><meta charset=""#, encoding, r#"">"#]);
    self.push_str(r#"<meta http-equiv="X-UA-Compatible" content="IE=edge">"#);
    self.push_str(r#"<meta name="viewport" content="width=device-width, initial-scale=1.0">"#);
    if !attrs.is_set("reproducible") {
      self.push_str(r#"<meta name="generator" content="Asciidork">"#);
    }
    if let Some(appname) = attrs.str("app-name") {
      self.push([r#"<meta name="application-name" content=""#, appname, "\">"]);
    }
    if let Some(desc) = attrs.str("description") {
      self.push([r#"<meta name="description" content=""#, desc, "\">"]);
    }
    if let Some(keywords) = attrs.str("keywords") {
      self.push([r#"<meta name="keywords" content=""#, keywords, "\">"]);
    }
    if let Some(copyright) = attrs.str("copyright") {
      self.push([r#"<meta name="copyright" content=""#, copyright, "\">"]);
    }
    self.render_favicon(attrs);
    self.render_authors(&document.header);
    self.render_title(document, attrs);
    // TODO: stylesheets
    self.push_str(r#"</head><body>"#);
  }

  fn exit_document(&mut self, _document: &Document, _header_attrs: &AttrEntries) {
    if !self.footnotes.is_empty() {
      self.render_footnotes();
    }
    if !self.flags.embedded {
      self.push_str("</body></html>");
    }
  }

  fn enter_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  fn exit_compound_block_content(&mut self, _children: &[Block], _block: &Block) {}
  fn enter_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {}
  fn exit_simple_block_content(&mut self, _children: &[InlineNode], _block: &Block) {}

  fn enter_sidebar_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["sidebarblock"], &block.attrs);
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_sidebar_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  fn enter_quoted_paragraph(&mut self, block: &Block, _attr: &str, _cite: Option<&str>) {
    self.open_element("div", &["quoteblock"], &block.attrs);
    self.visit_block_title(block.title.as_deref(), None);
    self.push_str("<blockquote>");
  }

  fn enter_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["quoteblock"], &block.attrs);
    self.visit_block_title(block.title.as_deref(), None);
    self.push_str("<blockquote>");
  }

  fn exit_quoted_paragraph(&mut self, _block: &Block, attr: &str, cite: Option<&str>) {
    self.exit_blockquote(Some(attr), cite);
  }

  fn exit_quote_block(&mut self, block: &Block, _content: &BlockContent) {
    if let Some(attrs) = &block.attrs {
      self.exit_blockquote(attrs.str_positional_at(1), attrs.str_positional_at(2));
    } else {
      self.exit_blockquote(None, None);
    }
  }

  fn enter_example_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["exampleblock"], &block.attrs);
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_example_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  fn enter_open_block(&mut self, block: &Block, _content: &BlockContent) {
    self.open_element("div", &["openblock"], &block.attrs);
    self.push_str(r#"<div class="content">"#);
  }

  fn exit_open_block(&mut self, _block: &Block, _content: &BlockContent) {
    self.push_str("</div></div>");
  }

  fn enter_unordered_list(&mut self, block: &Block, items: &[ListItem], _depth: u8) {
    let attrs = block.attrs.as_ref();
    let custom = attrs.and_then(|attrs| attrs.unordered_list_custom_marker_style());
    let interactive = attrs.map(|a| a.has_option("interactive")).unwrap_or(false);
    self.list_stack.push(interactive);
    let mut wrap_classes = SmallVec::<[&str; 3]>::from_slice(&["ulist"]);
    let mut list_classes = SmallVec::<[&str; 2]>::new();
    if let Some(custom) = custom {
      wrap_classes.push(custom);
      list_classes.push(custom);
    }
    if items.iter().any(|item| item.checklist.is_some()) {
      wrap_classes.push("checklist");
      list_classes.push("checklist");
    }
    self.open_element("div", &wrap_classes, &block.attrs);
    self.visit_block_title(block.title.as_deref(), None);
    self.push_str("<ul");
    self.add_classes(&list_classes);
    self.push_ch('>');
  }

  fn exit_unordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ul></div>");
  }

  fn enter_ordered_list(&mut self, block: &Block, items: &[ListItem], depth: u8) {
    let attrs = block.attrs.as_ref();
    self.list_stack.push(false);
    let custom = attrs.and_then(|attrs| attrs.ordered_list_custom_number_style());
    let list_type = custom
      .and_then(list_type_from_class)
      .unwrap_or_else(|| list_type_from_depth(depth));
    let class = custom.unwrap_or_else(|| list_class_from_depth(depth));
    let classes = &["olist", class];
    self.open_element("div", classes, &block.attrs);
    self.visit_block_title(block.title.as_deref(), None);
    self.push([r#"<ol class=""#, class, "\""]);

    if list_type != "1" {
      self.push([" type=\"", list_type, "\""]);
    }

    if let Some(attr_start) = attrs.and_then(|attrs| attrs.named("start")) {
      self.push([" start=\"", attr_start, "\""]);
    } else {
      match items[0].marker {
        ListMarker::Digits(1) => {}
        ListMarker::Digits(n) => {
          // TODO: asciidoctor documents that this is OK,
          // but it doesn't actually work, and emits a warning
          let digits_start = n.to_string();
          self.push([" start=\"", &digits_start, "\""]);
        }
        _ => {}
      }
    }

    if attrs.map_or(false, |attrs| attrs.has_option("reversed")) {
      self.push_str(" reversed>");
    } else {
      self.push_str(">");
    }
  }

  fn exit_ordered_list(&mut self, _block: &Block, _items: &[ListItem], _depth: u8) {
    self.list_stack.pop();
    self.push_str("</ol></div>");
  }

  fn enter_list_item_principal(&mut self, item: &ListItem) {
    self.push_str("<li><p>");
    self.render_checklist_item(item);
  }

  fn exit_list_item_principal(&mut self, _item: &ListItem) {
    self.push_str("</p>");
  }

  fn enter_list_item_blocks(&mut self, _blocks: &[Block], _item: &ListItem) {}

  fn exit_list_item_blocks(&mut self, _blocks: &[Block], _item: &ListItem) {
    self.push_str("</li>");
  }

  fn enter_paragraph_block(&mut self, block: &Block) {
    self.push_str(r#"<div class="paragraph">"#);
    self.visit_block_title(block.title.as_deref(), None);
    self.push_str("<p>");
  }

  fn exit_paragraph_block(&mut self, _block: &Block) {
    self.push_str("</p></div>");
  }

  fn enter_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("<em>");
  }

  fn exit_inline_italic(&mut self, _children: &[InlineNode]) {
    self.push_str("</em>");
  }

  fn visit_inline_text(&mut self, text: &str) {
    self.push_str(text);
  }

  fn visit_joining_newline(&mut self) {
    self.push_ch(' ');
  }

  fn enter_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("<code>");
  }

  fn exit_inline_mono(&mut self, _children: &[InlineNode]) {
    self.push_str("</code>");
  }

  fn enter_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("<strong>");
  }

  fn exit_inline_bold(&mut self, _children: &[InlineNode]) {
    self.push_str("</strong>");
  }

  fn enter_inline_passthrough(&mut self, _children: &[InlineNode]) {}
  fn exit_inline_passthrough(&mut self, _children: &[InlineNode]) {}

  fn visit_button_macro(&mut self, text: &str) {
    self.push([r#"<b class="button">"#, text, "</b>"])
  }

  fn visit_menu_macro(&mut self, items: &[&str]) {
    let mut items = items.iter();
    self.push_str(r#"<span class="menuseq"><span class="menu">"#);
    self.push_str(items.next().unwrap());
    self.push_str("</span>");

    let last_idx = items.len() - 1;
    for (idx, item) in items.enumerate() {
      self.push_str(r#"&#160;&#9656;<span class=""#);
      if idx == last_idx {
        self.push(["menuitem\">", item, "</span>"]);
      } else {
        self.push(["submenu\">", item, "</span>"]);
      }
    }
    self.push_str("</span>");
  }

  fn visit_inline_specialchar(&mut self, char: &SpecialCharKind) {
    match char {
      SpecialCharKind::Ampersand => self.push_str("&amp;"),
      SpecialCharKind::LessThan => self.push_str("&lt;"),
      SpecialCharKind::GreaterThan => self.push_str("&gt;"),
    }
  }

  fn enter_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("<mark>");
  }

  fn exit_inline_highlight(&mut self, _children: &[InlineNode]) {
    self.push_str("</mark>");
  }

  fn enter_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sub>");
  }

  fn exit_inline_subscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sub>");
  }

  fn enter_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("<sup>");
  }

  fn exit_inline_superscript(&mut self, _children: &[InlineNode]) {
    self.push_str("</sup>");
  }

  fn enter_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8220;"),
      QuoteKind::Single => self.push_str("&#8216;"),
    }
  }

  fn exit_inline_quote(&mut self, kind: QuoteKind, _children: &[InlineNode]) {
    match kind {
      QuoteKind::Double => self.push_str("&#8221;"),
      QuoteKind::Single => self.push_str("&#8217;"),
    }
  }

  fn visit_curly_quote(&mut self, kind: CurlyKind) {
    match kind {
      CurlyKind::LeftDouble => self.push_str("&#8221;"),
      CurlyKind::RightDouble => self.push_str("&#8220;"),
      CurlyKind::LeftSingle => self.push_str("&#8217;"),
      CurlyKind::RightSingle => self.push_str("&#8216;"),
      CurlyKind::LegacyImplicitApostrophe => self.push_str("&#8217;"),
    }
  }

  fn visit_inline_lit_mono(&mut self, text: &str) {
    self.push(["<code>", text, "</code>"]);
  }

  fn visit_multichar_whitespace(&mut self, _whitespace: &str) {
    self.push_ch(' ');
  }

  fn enter_admonition_block(&mut self, kind: AdmonitionKind, block: &Block) {
    let classes = &["admonitionblock", kind.lowercase_str()];
    self.open_element("div", classes, &block.attrs);
    self.push_str(r#"<table><tr><td class="icon"><div class="title">"#);
    self.push_str(kind.str());
    self.push_str(r#"</div></td><td class="content">"#);
    self.visit_block_title(block.title.as_deref(), None);
  }

  fn exit_admonition_block(&mut self, _kind: AdmonitionKind, _block: &Block) {
    self.push_str(r#"</td></tr></table></div>"#);
  }

  fn enter_image_block(&mut self, img_target: &str, img_attrs: &AttrList, block: &Block) {
    let alt = img_attrs.str_positional_at(0).unwrap_or({
      if let Some(captures) = REMOVE_FILE_EXT.captures(img_target) {
        captures.get(1).unwrap().as_str()
      } else {
        img_target
      }
    });
    self.open_element("div", &["imageblock"], &block.attrs);
    self.push_str(r#"<div class="content">"#);
    let mut has_link = false;
    if let Some(href) = &block.attrs.as_ref().and_then(|attrs| attrs.named("link")) {
      self.push([r#"<a class="image" href=""#, *href, r#"">"#]);
      has_link = true;
    }
    self.push([r#"<img src=""#, img_target, r#"" alt=""#, alt, "\""]);
    if let Some(width) = img_attrs.str_positional_at(1) {
      self.push([r#" width=""#, width, "\""]);
    }
    if let Some(height) = img_attrs.str_positional_at(2) {
      self.push([r#" height=""#, height, "\""]);
    }
    self.push_ch('>');
    if has_link {
      self.push_str("</a>");
    }
    self.push_str(r#"</div>"#);
  }

  fn exit_image_block(&mut self, block: &Block) {
    let prefix = if self.doc_attrs.is_unset("figure-caption") {
      None
    } else {
      self.fig_caption_num += 1;
      Some(Cow::Owned(format!("Figure {}. ", self.fig_caption_num)))
    };
    self.visit_block_title(block.title.as_deref(), prefix);
    self.push_str(r#"</div>"#);
  }

  fn visit_document_attribute_decl(&mut self, name: &str, entry: &AttrEntry) {
    self.doc_attrs.insert(name.to_string(), entry.clone());
  }

  fn enter_footnote(&mut self, _id: Option<&str>, _content: &[InlineNode]) {
    mem::swap(&mut self.html, &mut self.alt_html);
  }

  fn exit_footnote(&mut self, id: Option<&str>, _content: &[InlineNode]) {
    mem::swap(&mut self.alt_html, &mut self.html);
    let mut footnote = String::new();
    mem::swap(&mut footnote, &mut self.alt_html);
    let num = (self.footnotes.len() + 1).to_string();
    self.push_str(r#"<sup class="footnote""#);
    if let Some(id) = id {
      self.push([r#" id="_footnote_"#, id, "\""]);
    }
    self.push_str(r#">[<a id="_footnoteref_"#);
    self.push([&num, r##"" class="footnote" href="#_footnotedef_"##, &num]);
    self.push([r#"" title="View footnote.">"#, &num, "</a>]</sup>"]);
    let id = id.unwrap_or(&num);
    self.footnotes.push((id.to_string(), footnote));
  }

  fn into_result(self) -> Result<Self::Output, Self::Error> {
    Ok(self.html)
  }

  fn result(&self) -> Result<&Self::Output, Self::Error> {
    Ok(&self.html)
  }
}

impl AsciidoctorHtml {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn into_string(self) -> String {
    self.html
  }

  fn push_str(&mut self, s: &str) {
    self.html.push_str(s);
  }

  fn push_ch(&mut self, c: char) {
    self.html.push(c);
  }

  fn push<const N: usize>(&mut self, strs: [&str; N]) {
    for s in strs {
      self.push_str(s);
    }
  }

  fn visit_block_title(&mut self, title: Option<&str>, prefix: Option<Cow<str>>) {
    if let Some(title) = title {
      self.push_str(r#"<div class="title">"#);
      if let Some(prefix) = prefix {
        self.push_str(prefix.as_ref());
      }
      self.push_str(title);
      self.push_str("</div>");
    }
  }

  fn open_element(&mut self, element: &str, classes: &[&str], attrs: &Option<AttrList>) {
    self.push_ch('<');
    self.push_str(element);
    if let Some(id) = attrs.as_ref().and_then(|a| a.id.as_ref()) {
      self.push_str(" id=\"");
      self.push_str(id);
      self.push_ch('"');
    }
    if !classes.is_empty() || attrs.as_ref().map_or(false, |a| !a.roles.is_empty()) {
      self.push_str(" class=\"");
      for class in classes {
        self.push_str(class);
        self.push_ch(' ');
      }
      if let Some(roles) = attrs.as_ref().map(|a| &a.roles) {
        for role in roles {
          self.push_str(role);
          self.push_ch(' ');
        }
      }
      self.html.pop();
      self.push_ch('"');
    }
    self.push_ch('>');
  }

  fn render_footnotes(&mut self) {
    self.push_str(r#"<div id="footnotes"><hr>"#);
    let footnotes = mem::take(&mut self.footnotes);
    for (index, (_id, footnote)) in footnotes.iter().enumerate() {
      let num = &(index + 1).to_string();
      self.push_str(r#"<div class="footnote" id="_footnotedef_"#);
      self.push([num, r##""><a href="#_footnoteref_"##, num, "\">"]);
      self.push([num, "</a>. ", footnote, "</div>"]);
    }
    self.push_str(r#"</div>"#);
    self.footnotes = footnotes;
  }

  fn render_favicon(&mut self, attrs: &AttrEntries) {
    match attrs.get("favicon") {
      Some(AttrEntry::String(path)) => {
        let ext = helpers::file_ext(path).unwrap_or("ico");
        self.push_str(r#"<link rel="icon" type="image/"#);
        self.push([ext, r#"" href=""#, path, "\">"]);
      }
      Some(AttrEntry::Bool(true)) => {
        self.push_str(r#"<link rel="icon" type="image/x-icon" href="favicon.ico">"#);
      }
      _ => {}
    }
  }

  fn render_authors(&mut self, header: &Option<DocHeader>) {
    let Some(DocHeader { authors, .. }) = header else {
      return;
    };
    if authors.is_empty() {
      return;
    }
    self.push_str(r#"<meta name="author" content=""#);
    for (index, author) in authors.iter().enumerate() {
      if index > 0 {
        self.push_str(", ");
      }
      // TODO: escape/sanitize, w/ tests, see asciidoctor
      self.push_str(&author.fullname());
    }
    self.push_str(r#"">"#);
  }

  fn render_title(&mut self, document: &Document, attrs: &AttrEntries) {
    self.push_str(r#"<title>"#);
    if let Some(title) = attrs.str("title") {
      self.push_str(title);
    } else if let Some(title) = document
      .header
      .as_ref()
      .and_then(|header| header.title.as_ref())
    {
      for s in title.heading.plain_text() {
        self.push_str(s);
      }
    } else {
      self.push_str("Untitled");
    }
    self.push_str(r#"</title>"#);
  }

  fn exit_blockquote(&mut self, attribution: Option<&str>, cite: Option<&str>) {
    self.push_str("</blockquote>");
    if let Some(attribution) = attribution {
      self.push_str(r#"<div class="attribution">&#8212; "#);
      self.push_str(attribution);
      if let Some(cite) = cite {
        self.push_str(r#"<br><cite>"#);
        self.push([cite, "</cite>"]);
      }
      self.push_str("</div>");
    } else if let Some(cite) = cite {
      self.push_str(r#"<div class="attribution">&#8212; "#);
      self.push([cite, "</div>"]);
    }
    self.push_str("</div>");
  }

  fn add_classes(&mut self, classes: &[&str]) {
    if !classes.is_empty() {
      self.push_str(" class=\"");
      for class in classes.iter().take(classes.len() - 1) {
        self.push_str(class);
        self.push_ch(' ');
      }
      self.push_str(classes.last().unwrap());
      self.push_ch('"');
    }
  }

  fn render_checklist_item(&mut self, item: &ListItem) {
    if let Some((checked, _)) = &item.checklist {
      match (self.list_stack.last() == Some(&true), checked) {
        (false, true) => self.push_str("&#10003;"),
        (false, false) => self.push_str("&#10063;"),
        (true, true) => self.push_str(r#"<input type="checkbox" data-item-complete="1" checked>"#),
        (true, false) => self.push_str(r#"<input type="checkbox" data-item-complete="0">"#),
      }
    }
  }
}

fn list_type_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "1",
    2 => "a",
    3 => "i",
    4 => "A",
    _ => "I",
  }
}

fn list_type_from_class(class: &str) -> Option<&'static str> {
  match class {
    "arabic" => Some("1"),
    "loweralpha" => Some("a"),
    "lowerroman" => Some("i"),
    "upperalpha" => Some("A"),
    "upperroman" => Some("I"),
    _ => None,
  }
}

fn list_class_from_depth(depth: u8) -> &'static str {
  match depth {
    1 => "arabic",
    2 => "loweralpha",
    3 => "lowerroman",
    4 => "upperalpha",
    _ => "upperroman",
  }
}

lazy_static! {
  pub static ref REMOVE_FILE_EXT: Regex = Regex::new(r"^(.*)\.[^.]+$").unwrap();
}
