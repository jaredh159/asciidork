use asciidork_core::{iff, regx};
use ast::{AttrData, AttrList, ReadAttr};

use crate::html::backend::HtmlBackend;

pub(crate) fn element<B: HtmlBackend + ?Sized>(backend: &mut B, target: &str, attrs: &AttrList) {
  let poster = attrs.str_positional_at(0);
  match poster {
    Some("youtube") => service(backend, Provider::YouTube, target, attrs),
    Some("vimeo") => service(backend, Provider::Vimeo, target, attrs),
    Some("wistia") => service(backend, Provider::Wistia, target, attrs),
    _ => native_element(backend, target, attrs, attrs.named("poster").or(poster)),
  }
}

fn service<B: HtmlBackend + ?Sized>(
  backend: &mut B,
  provider: Provider,
  target: &str,
  attrs: &AttrList,
) {
  backend.push_str("<iframe");
  push_dims(backend, attrs);
  backend.push([" src=\"", provider.url_start()]);
  let mut param_pushed = false;

  let target_parts = target.split_once("/");
  if let Some(param) = provider.slash_param()
    && let Some((id, hash)) = target_parts
  {
    backend.push([id, "?", param, "=", hash]);
    param_pushed = true;
  } else if provider == Provider::YouTube && target.contains(",") {
    for (index, id) in target.split(",").enumerate() {
      if index == 0 {
        backend.push([id, "?playlist=", id]);
        param_pushed = true;
      } else {
        backend.push([",", id]);
      }
    }
  } else if provider == Provider::YouTube
    && let Some(playlist) = attrs.named("playlist")
  {
    backend.push([target, "?playlist=", target, ",", playlist]);
    param_pushed = true;
  } else {
    backend.push_str(target);
    if let Some(hash) = attrs.named("hash") {
      backend.push([iff!(param_pushed, "&amp;", "?"), "h=", hash]);
      param_pushed = true;
    }
  }

  for (opt, key) in provider.simple_param_pairs() {
    if attrs.has_option(opt) {
      backend.push([iff!(param_pushed, "&amp;", "?"), key]);
      param_pushed = true;
    }
  }

  if provider == Provider::YouTube {
    more_youtube_params(backend, attrs, target, &mut param_pushed);
  }

  // start param (last because vimeo uses # frag not &)
  if let Some(start) = attrs.named("start") {
    match provider {
      Provider::YouTube => backend.push([iff!(param_pushed, "&amp;", "?"), "start=", start]),
      Provider::Wistia => backend.push([iff!(param_pushed, "&amp;", "?"), "time=", start]),
      Provider::Vimeo => backend.push(["#at=", start]),
    }
  }

  backend.push_str("\" frameborder=\"0\"");
  if !attrs.has_option("nofullscreen") {
    backend.push_str(" allowfullscreen");
  }

  if provider == Provider::Wistia {
    backend.push_str(r#" class="wistia_embed" name="wistia_embed""#);
  }

  backend.push_ch('>');
  backend.push_str("</iframe>");
}

fn native_element<B: HtmlBackend + ?Sized>(
  backend: &mut B,
  target: &str,
  attrs: &AttrList,
  poster: Option<&str>,
) {
  let imagesdir = backend.doc_meta().string("imagesdir");
  backend.push_str(r#"<video src=""#);
  if !regx::URI_SNIFF.is_match(target)
    && let Some(ref imagesdir) = imagesdir
  {
    backend.push([imagesdir, "/"]);
  }
  backend.push_str(target);
  let start = attrs.named("start");
  let end = attrs.named("end");
  match (start, end) {
    (None, Some(end)) => backend.push(["#t=,", end, "\""]),
    (Some(start), None) => backend.push(["#t=", start, "\""]),
    (Some(start), Some(end)) => backend.push(["#t=", start, ",", end, "\""]),
    (None, None) => backend.push_ch('"'),
  }
  push_dims(backend, attrs);
  if let Some(poster) = poster {
    if !regx::URI_SNIFF.is_match(poster)
      && let Some(ref imagesdir) = imagesdir
    {
      backend.push([" poster=\"", imagesdir, "/", poster, "\""]);
    } else {
      backend.push_html_attr("poster", poster);
    }
  }
  if attrs.has_option("autoplay") {
    backend.push_str(" autoplay");
  }
  if attrs.has_option("muted") {
    backend.push_str(" muted");
  }
  if attrs.has_option("loop") {
    backend.push_str(" loop");
  }
  if let Some(preload) = attrs.named("preload") {
    backend.push_html_attr("preload", preload);
  }
  if !attrs.has_option("nocontrols") {
    backend.push_str(" controls");
  }
  backend.push_str(">Your browser does not support the video tag.</video>");
}

fn more_youtube_params<B: HtmlBackend + ?Sized>(
  backend: &mut B,
  attrs: &AttrList,
  target: &str,
  param_pushed: &mut bool,
) {
  backend.push([
    iff!(*param_pushed, "&amp;rel=", "?rel="),
    iff!(attrs.has_option("related"), "1", "0"),
  ]);
  *param_pushed = true;
  if let Some(theme) = attrs.named("theme") {
    backend.push([iff!(*param_pushed, "&amp;theme=", "?theme="), theme]);
    *param_pushed = true;
  }
  if let Some(lang) = attrs.named("lang") {
    backend.push([iff!(*param_pushed, "&amp;hl=", "?hl="), lang]);
    *param_pushed = true;
  }
  if let Some(end) = attrs.named("end") {
    backend.push([iff!(*param_pushed, "&amp;end=", "?end="), end]);
    *param_pushed = true;
  }
  if attrs.has_option("nofullscreen") {
    backend.push_str(iff!(*param_pushed, "&amp;fs=0", "?fs=0"));
    *param_pushed = true;
  }
  if attrs.has_option("nocontrols") {
    backend.push_str(iff!(*param_pushed, "&amp;controls=0", "?controls=0"));
    *param_pushed = true;
  }
  if let Some(list) = attrs.named("list") {
    backend.push([iff!(*param_pushed, "&amp;list=", "?list="), list]);
    *param_pushed = true;
  } else if attrs.has_option("loop") && !target.contains(['/', ',']) {
    backend.push([iff!(*param_pushed, "&amp;", "?"), "playlist=", target]);
    *param_pushed = true;
  }
}

fn push_dims<B: HtmlBackend + ?Sized>(backend: &mut B, attrs: &AttrList) {
  if let Some(width) = attrs.named("width") {
    backend.push_html_attr("width", width);
  } else if let Some(width) = attrs.str_positional_at(1) {
    backend.push_html_attr("width", width);
  }
  if let Some(height) = attrs.named("height") {
    backend.push_html_attr("height", height);
  } else if let Some(height) = attrs.str_positional_at(2) {
    backend.push_html_attr("height", height);
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
  YouTube,
  Vimeo,
  Wistia,
}

impl Provider {
  const fn url_start(&self) -> &'static str {
    match self {
      Provider::YouTube => "https://www.youtube.com/embed/",
      Provider::Vimeo => "https://player.vimeo.com/video/",
      Provider::Wistia => "https://fast.wistia.com/embed/iframe/",
    }
  }

  const fn slash_param(&self) -> Option<&str> {
    match self {
      Provider::YouTube => Some("list"),
      Provider::Vimeo => Some("h"),
      Provider::Wistia => None,
    }
  }

  const fn simple_param_pairs(&self) -> &[(&'static str, &'static str)] {
    match self {
      Provider::YouTube => &[
        ("autoplay", "autoplay=1"),
        ("muted", "mute=1"),
        ("modest", "modestbranding=1"),
        ("loop", "loop=1"),
      ],
      Provider::Vimeo => &[
        ("autoplay", "autoplay=1"),
        ("muted", "muted=1"),
        ("loop", "loop=1"),
      ],
      Provider::Wistia => &[
        ("autoplay", "autoPlay=true"),
        ("muted", "muted=true"),
        ("loop", "endVideoBehavior=loop"),
        ("reset", "endVideoBehavior=reset"),
      ],
    }
  }
}
