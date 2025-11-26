use asciidork_ast::Document;
use asciidork_core::{Path, ReadAttr, SafeMode};

pub fn resolve(document: &mut Document) -> Result<(), String> {
  let attrs = document.meta.header_attrs();
  let Some(custom_filename) = attrs.str("stylesheet") else {
    return Ok(());
  };
  if custom_filename.is_empty() {
    return Ok(());
  }

  let stylesdir = attrs.str_or("stylesdir", ".");
  let cwd = std::env::current_dir().expect("invalid cwd");
  let mut path: Path = cwd.clone().into();
  path.push(stylesdir);
  path.push(custom_filename);

  let canonical_path = dunce::canonicalize(path.to_string()).map_err(|_| {
    format!(
      "stylesheet `{}{}{}` not found",
      stylesdir,
      std::path::MAIN_SEPARATOR_STR,
      custom_filename
    )
  })?;

  if document.meta.safe_mode != SafeMode::Unsafe && !canonical_path.starts_with(cwd) {
    return Err(format!(
      "stylesheet path `{}{}{}` not permitted outside cwd except in unsafe mode",
      stylesdir,
      std::path::MAIN_SEPARATOR_STR,
      custom_filename,
    ));
  }

  let css = std::fs::read_to_string(canonical_path).map_err(|err| {
    format!(
      "reading file `{}{}{}`: {}",
      stylesdir,
      std::path::MAIN_SEPARATOR_STR,
      custom_filename,
      err
    )
  })?;

  document
    .meta
    .insert_header_attr("_asciidork_resolved_custom_css", css)
    .unwrap();
  Ok(())
}
