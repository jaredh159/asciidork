/// NB: does not return the `.`
pub fn ext(input: &str) -> Option<&str> {
  let basename = basename(input);
  if let Some(idx) = basename.rfind('.') {
    Some(&basename[idx + 1..])
  } else {
    None
  }
}

pub fn has_ext(input: &str) -> bool {
  basename(input).rfind('.').is_some()
}

pub fn has_adoc_ext(path: &str) -> bool {
  matches!(
    ext(path),
    Some("adoc") | Some("asciidoc") | Some("asc") | Some("ad")
  )
}

pub fn remove_ext(input: &str) -> &str {
  if !has_ext(input) {
    input
  } else {
    let idx = input.rfind('.').unwrap();
    &input[..idx]
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
