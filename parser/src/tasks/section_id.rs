use lazy_static::lazy_static;
use regex::Regex;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn section_id(
    &mut self,
    line: &Line<'arena>,
    attrs: &MultiAttrList<'arena>,
  ) -> Option<BumpString<'arena>> {
    if self.document.meta.is_false("sectids") {
      return None;
    }
    if let Some(id) = attrs.id() {
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
    let auto_gen_id = self.autogen_sect_id(&line.reassemble_src(), id_prefix, id_sep);
    self.ctx.anchor_ids.borrow_mut().insert(auto_gen_id.clone());
    Some(auto_gen_id)
  }

  /// @see https://docs.asciidoctor.org/asciidoc/latest/sections/auto-ids/#how-a-section-id-is-computed
  fn autogen_sect_id(
    &self,
    line: &str,
    prefix: &str,
    separator: Option<char>,
  ) -> BumpString<'arena> {
    self._autogen_sect_id(line, prefix, separator, false, false)
  }

  fn _autogen_sect_id(
    &self,
    line: &str,
    prefix: &str,
    separator: Option<char>,
    removed_entities: bool,
    removed_aux_ids: bool,
  ) -> BumpString<'arena> {
    let mut id = BumpString::with_capacity_in(line.len() + prefix.len() + 3, self.bump);
    let mut in_html_tag = false;
    let mut last_c = prefix.chars().last().unwrap_or('\0');
    id.push_str(prefix);

    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
      match c {
        '<' if chars.peek() == Some(&'<') => {
          chars.next();
        }
        '<' => in_html_tag = true,
        '>' => in_html_tag = false,
        ' ' | '-' | '.' | ',' | '\t' => {
          if separator.map(|c| c != last_c).unwrap_or(false) {
            id.push(separator.unwrap());
            last_c = separator.unwrap();
          }
        }
        // only pay the cost of the regex if we need to
        '[' if chars.peek() == Some(&'[') && !removed_aux_ids => {
          let sans_aux_ids = AUX_ID_RE.replace_all(line, "$1");
          return self._autogen_sect_id(&sans_aux_ids, prefix, separator, removed_entities, true);
        }
        '&' if !removed_entities => {
          let sans_entities = ENTITY_RE.replace_all(line, "");
          return self._autogen_sect_id(&sans_entities, prefix, separator, true, removed_aux_ids);
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

    if separator.is_some() && id.is_empty() {
      return self.sequence_sectid(&id, separator);
    }

    if prefix.is_empty() && separator.map(|c| id.starts_with(c)).unwrap_or(false) {
      id = self.string(&id[1..]);
    }

    if self.ctx.anchor_ids.borrow().contains(&id) {
      return self.sequence_sectid(&id, separator);
    }

    id
  }

  fn sequence_sectid(&self, id: &str, separator: Option<char>) -> BumpString<'arena> {
    let mut i = 2;
    loop {
      let mut sequenced = BumpString::with_capacity_in(id.len() + 2, self.bump);
      sequenced.push_str(id);
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
}

lazy_static! {
  static ref ENTITY_RE: Regex = Regex::new(
    r"&(?:[A-Za-z][A-Za-z]+\d{0,2}|#\d\d\d{0,4}|#x[\dA-Fa-f][\dA-Fa-f][\dA-Fa-f]{0,3});"
  )
  .unwrap();
}

lazy_static! {
  static ref AUX_ID_RE: Regex = Regex::new(r"\[\[.*?\]\]").unwrap();
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
      let id = parser.autogen_sect_id(input, "_", Some('_'));
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
      ("<em></em>", "_", Some('_'), &[], "_2"),
      // ignores auxiliary ids
      ("[[aux-id]]foo Bar", "_", Some('_'), &[], "_foo_bar"),
      ("[[aux-id]][[two]]foo Bar", "_", Some('_'), &[], "_foo_bar"),
      ("foo Bar[[aux-id]][[two]]", "_", Some('_'), &[], "_foo_bar"),
    ];

    for (line, id_prefix, id_sep, prev, expected) in cases {
      let parser = test_parser!("");
      for s in prev {
        parser.ctx.anchor_ids.borrow_mut().insert(bstr!(s));
      }
      let id = parser.autogen_sect_id(line, id_prefix, id_sep);
      assert_eq!(id, *expected);
    }
  }
}
