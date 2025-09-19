use crate::{html::HtmlBuf, Backend};
use ast::ReadAttr;

// TODO: handle embedding images, data-uri, etc., this is a naive impl
// @see https://github.com/jaredh159/asciidork/issues/7
pub fn push_icon_uri<B>(backend: &mut B, name: &str, prefix: Option<&str>)
where
  B: Backend + HtmlBuf,
{
  // PERF: we could work to prevent all these allocations w/ some caching
  // these might get rendered many times in a given document
  let icondir = backend.doc_meta().string_or("iconsdir", "./images/icons");
  let ext = backend.doc_meta().string_or("icontype", "png");
  backend.push([&icondir, "/", prefix.unwrap_or(""), name, ".", &ext]);
}
