use std::collections::HashSet;

use ast::AttrEntry;
use lazy_static::lazy_static;
use regex::Regex;

/// @see https://docs.asciidoctor.org/asciidoc/latest/sections/auto-ids/#how-a-section-id-is-computed
pub fn autogenerate_id(
  html: &str,
  id_prefix: &str,
  id_sep: Option<char>,
  previous_ids: &HashSet<String>,
) -> String {
  autogenerate_id_impl(html, id_prefix, id_sep, previous_ids, false)
}

fn autogenerate_id_impl(
  html: &str,
  prefix: &str,
  separator: Option<char>,
  previous_ids: &HashSet<String>,
  removed_entities: bool,
) -> String {
  let mut id = String::with_capacity(html.len() + prefix.len() + 3);
  let mut in_html_tag = false;
  let mut last_c = prefix.chars().last().unwrap_or('\0');
  id.push_str(prefix);

  for c in html.chars() {
    match c {
      '<' => in_html_tag = true,
      '>' => in_html_tag = false,
      ' ' | '-' | '.' | ',' | '\t' => {
        if separator.map(|c| c != last_c).unwrap_or(false) {
          id.push(separator.unwrap());
          last_c = separator.unwrap();
        }
      }
      // only pay the cost of the hairy regex if we encounter an ampersand
      '&' if !removed_entities => {
        let sans_entities = ENTITY_RE.replace_all(html, "");
        return autogenerate_id_impl(&sans_entities, prefix, separator, previous_ids, true);
      }
      _ if in_html_tag => {}
      mut c if c.is_ascii_alphanumeric() => {
        c.make_ascii_lowercase();
        last_c = c;
        id.push(c);
      }
      c if c.is_alphanumeric() => {
        c.to_lowercase().for_each(|c| {
          last_c = c;
          id.push(c);
        });
      }
      _ => {}
    }
  }

  // not documented, but asciidoctor does this
  if Some(last_c) == separator {
    id.pop();
  }

  if prefix.is_empty() && separator.map(|c| id.starts_with(c)).unwrap_or(false) {
    id = id[1..].to_string();
  }

  if previous_ids.contains(&id) {
    let mut i = 2;
    loop {
      let sequenced = format!(
        "{}{}{}",
        id,
        separator.map(|c| c.to_string()).unwrap_or("".to_string()),
        i
      );
      if !previous_ids.contains(&sequenced) {
        return sequenced;
      }
      i += 1;
    }
  }

  id
}

pub fn id_separator(doc_attrs: &ast::AttrEntries) -> Option<char> {
  match doc_attrs.get("idseparator") {
    Some(AttrEntry::Bool(true)) => None,
    Some(AttrEntry::String(s)) => s.chars().next(),
    _ => Some('_'),
  }
}

pub fn id_prefix(doc_attrs: &ast::AttrEntries) -> &str {
  match doc_attrs.get("idprefix") {
    Some(AttrEntry::Bool(true)) => "",
    Some(AttrEntry::String(s)) => s,
    _ => "_",
  }
}

lazy_static! {
  static ref ENTITY_RE: Regex = Regex::new(
    r"&(?:[A-Za-z][A-Za-z]+\d{0,2}|#\d\d\d{0,4}|#x[\dA-Fa-f][\dA-Fa-f][\dA-Fa-f]{0,3});"
  )
  .unwrap();
}

// tests

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! test_sect_id {
    ($name:ident, $html:expr, $expected:expr) => {
      #[test]
      fn $name() {
        assert_eq!(
          autogenerate_id($html, "_", Some('_'), &mut HashSet::new()),
          $expected
        );
      }
    };
  }

  test_sect_id!(
    removes_entities,
    "Ben & Jerry &amp; Company&sup1; &#34;Ice Cream Brothers&#34; &#12354;",
    "_ben_jerry_company_ice_cream_brothers"
  );

  test_sect_id!(
    mixed_case_adjacent_entities,
    "a &#xae;&AMP;&#xA9; b",
    "_a_b"
  );

  test_sect_id!(
    removes_xml_tags,
    "Use the <code>run</code> command to make it <span class=\"icon\">[gear]</span>",
    "_use_the_run_command_to_make_it_gear"
  );

  test_sect_id!(
    collapses_repeating_spaces,
    "     Go       Far   ",
    "_go_far"
  );

  test_sect_id!(
    replaces_hyphens_with_separator,
    "== State-of-the-art design",
    "_state_of_the_art_design"
  );

  test_sect_id!(
    replaces_dots_with_separator,
    "Section 1.1.1",
    "_section_1_1_1"
  );

  #[test]
  fn test_autogenerate_section_id() {
    #[allow(clippy::type_complexity)]
    let cases: Vec<(&str, &str, Option<char>, &[&str], &str)> = vec![
      ("foo Bar", "_", Some('_'), &[], "_foo_bar"),
      ("foo Bar", "id_", Some('-'), &[], "id_foo-bar"),
      ("foo Bar", "", Some('-'), &[], "foo-bar"),
      ("Section One", "_", None, &[], "_sectionone"),
      ("Section One", "", None, &[], "sectionone"),
      ("+Section One", "", Some('_'), &[], "section_one"),
      ("Foo, bar.", "_", Some('_'), &[], "_foo_bar"),
      ("foo bar", "_", Some('_'), &["_foo_bar"], "_foo_bar_2"),
      ("foo   ,.  bar", "_", Some('_'), &[], "_foo_bar"),
      ("foo <em>bar</em>", "_", Some('_'), &[], "_foo_bar"),
      ("We’re back!", "_", Some('_'), &[], "_were_back"),
      ("Section $ One", "_", Some('_'), &[], "_section_one"),
      ("& ! More", "", Some('_'), &[], "more"), // sep stripped from beginning
      ("Foo-bar design", "_", Some('-'), &[], "_foo-bar-design"),
      ("Version 5.0.1", "_", Some('.'), &[], "_version.5.0.1"),
    ];

    for (html, id_prefix, id_sep, prev, expected) in cases {
      let previous_ids = prev.iter().map(|s| s.to_string()).collect();
      assert_eq!(
        autogenerate_id(html, id_prefix, id_sep, &previous_ids),
        expected
      );
    }
  }
}
