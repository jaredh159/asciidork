use ast::Section;

pub fn section_class(section: &Section) -> &'static str {
  match section.level {
    0 => "sect0",
    1 => "sect1",
    2 => "sect2",
    3 => "sect3",
    4 => "sect4",
    5 => "sect5",
    6 => "sect6",
    _ => unreachable!("section::class() level={}", section.level),
  }
}

// tests

#[cfg(test)]
mod tests {
  use crate::html::{backend::*, HtmlBuf};

  struct TestBackend {
    html: String,
    state: BackendState,
    doc_meta: ast::DocumentMeta,
  }

  impl HtmlBuf for TestBackend {
    fn htmlbuf(&mut self) -> &mut String {
      &mut self.html
    }
  }

  impl HtmlBackend for TestBackend {
    fn state(&self) -> &BackendState {
      &self.state
    }

    fn state_mut(&mut self) -> &mut BackendState {
      &mut self.state
    }

    fn doc_meta(&self) -> &ast::DocumentMeta {
      &self.doc_meta
    }
  }

  #[test]
  fn test_number_prefix() {
    let cases = vec![
      (1, [0, 0, 0, 0, 0], "1. ", [1, 0, 0, 0, 0], false),
      (1, [1, 0, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
      (2, [1, 0, 0, 0, 0], "1.1. ", [1, 1, 0, 0, 0], false),
      (2, [1, 1, 0, 0, 0], "1.2. ", [1, 2, 0, 0, 0], false),
      (1, [1, 1, 0, 0, 0], "2. ", [2, 0, 0, 0, 0], false),
      (3, [2, 4, 0, 0, 0], "2.4.1. ", [2, 4, 1, 0, 0], false),
      (2, [1, 1, 0, 0, 0], "B.2. ", [1, 2, 0, 0, 0], true),
      (3, [1, 2, 0, 0, 0], "B.2.1. ", [1, 2, 1, 0, 0], true),
    ];
    for (level, sect_nums, expected, after_mutation, apndx) in cases {
      let mut backend = TestBackend {
        html: String::new(),
        state: BackendState::default(),
        doc_meta: ast::DocumentMeta::default(),
      };
      backend.state.section_nums = sect_nums;
      if apndx {
        backend.state.ephemeral.insert(EphemeralState::InAppendix);
      }
      backend.push_section_number_prefix(level);
      assert_eq!(&backend.html, expected);
      assert_eq!(backend.state.section_nums, after_mutation);
    }
  }
}
