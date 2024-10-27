use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn section_id(
    &mut self,
    line: &Line<'arena>,
    attrs: Option<&AttrList<'arena>>,
  ) -> Option<BumpString<'arena>> {
    if self.document.meta.is_false("sectids") {
      return None;
    }
    if let Some(id) = attrs.and_then(|a| a.id.as_ref()) {
      let custom_id = self.string(&id.src);
      self.ctx.anchor_ids.borrow_mut().insert(custom_id.clone());
      return Some(custom_id);
    }
    let id_sep = match self.document.meta.get("idseparator") {
      Some(AttrValue::Bool(true)) => None,
      Some(AttrValue::String(s)) => s.chars().next(),
      _ => Some('_'),
    };
    let id_prefix = match self.document.meta.get("idprefix") {
      Some(AttrValue::Bool(true)) => "",
      Some(AttrValue::String(s)) => s,
      _ => "_",
    };
    let auto_gen_id = self.autogen_sect_id(&line.reassemble_src(), id_prefix, id_sep, false);
    self.ctx.anchor_ids.borrow_mut().insert(auto_gen_id.clone());
    Some(auto_gen_id)
  }

  /// @see https://docs.asciidoctor.org/asciidoc/latest/sections/auto-ids/#how-a-section-id-is-computed
  fn autogen_sect_id(
    &self,
    line: &str,
    prefix: &str,
    separator: Option<char>,
    removed_entities: bool,
  ) -> BumpString<'arena> {
    let mut id = BumpString::with_capacity_in(line.len() + prefix.len() + 3, self.bump);
    let mut in_html_tag = false;
    let mut last_c = prefix.chars().last().unwrap_or('\0');
    id.push_str(prefix);

    for c in line.chars() {
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
          let sans_entities = ENTITY_RE.replace_all(line, "");
          return self.autogen_sect_id(&sans_entities, prefix, separator, true);
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
      id = self.string(&id[1..]);
    }

    if self.ctx.anchor_ids.borrow().contains(&id) {
      let mut i = 2;
      loop {
        let mut sequenced = BumpString::with_capacity_in(id.len() + 2, self.bump);
        sequenced.push_str(&id);
        if let Some(c) = separator {
          sequenced.push(c);
        }
        sequenced.push_str(&i.to_string());
        if !self.ctx.anchor_ids.borrow().contains(&sequenced) {
          return sequenced;
        }
        i += 1;
      }
    }

    id
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
  use test_utils::*;

  #[test]
  fn test_autogen_sect_id() {
    let cases = &[
      ("Ben & Jerry's Ice Cream!", "_ben_jerrys_ice_cream"),
      ("a &#xae;&AMP;&#xA9; b", "_a_b"),
      ("foo +++<em>+++bar+++</em>+++ <i>baz</i>", "_foo_bar_baz"),
      ("     Go       Far   ", "_go_far"),
      ("State-of-the-art design", "_state_of_the_art_design"),
      ("Section 1.1.1", "_section_1_1_1"),
    ];
    let parser = test_parser!("");
    for (input, expected) in cases {
      let id = parser.autogen_sect_id(input, "_", Some('_'), false);
      assert_eq!(id, *expected);
    }
  }

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

    for (line, id_prefix, id_sep, prev, expected) in cases {
      let parser = test_parser!("");
      for s in prev {
        parser.ctx.anchor_ids.borrow_mut().insert(bstr!(s));
      }
      let id = parser.autogen_sect_id(line, id_prefix, id_sep, false);
      assert_eq!(id, *expected);
    }
  }
}
