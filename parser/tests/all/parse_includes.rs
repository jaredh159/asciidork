use asciidork_ast::prelude::*;
use asciidork_core::SafeMode;
use asciidork_core::{JobAttr, JobSettings};
use asciidork_parser::includes::*;
use asciidork_parser::prelude::*;
use test_utils::*;

#[test]
fn trims_trailing_from_adoc() {
  assert_doc_content!(
    resolving: b"windows \r\nand unix \t \n",
    adoc! {"
      ----
      include::some_file.adoc[]
      ----
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Listing,
        content: BlockContent::Simple(nodes![
          node!("windows"; 0..7, depth: 1), // <-- trimmed
          node!(Inline::Newline, 7..8, depth: 1),
          node!("and unix"; 8..16, depth: 1), // <-- trimmed
        ]),
        ..empty_block!(0, 35)
      }
    ])
  );
}

#[test]
fn no_trim_trailing_non_adoc() {
  assert_doc_content!(
    resolving: b"text \n",
    adoc! {"
      ----
      include::some_file.rs[]
      ----
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Listing,
        content: BlockContent::Simple(nodes![
          node!("text "; 0..5, depth: 1), // <-- not trimmed
        ]),
        ..empty_block!(0, 33)
      }
    ])
  );
}

#[test]
fn include_no_trailing_newline() {
  assert_doc_content!(
    resolving: b"Line-2!",
    adoc! {"
      Line-1
      include::some_file.adoc[]
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!("Line-2!"; 0..7, depth: 1),
        ]),
        loc: MultiSourceLocation {
          start_pos: 0,
          start_depth: 0,
          end_pos: 7,
          end_depth: 1,
        },
        ..empty_block!(0)
      }
    ])
  );
  assert_doc_content!(
    resolving: b"Line-2!",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0, 39)
      }
    ])
  );
}

#[test]
fn include_with_trailing_newline() {
  assert_doc_content!(
    resolving: b"Line-2!\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!("Line-2!"; 0..7, depth: 1),
        ]),
        loc: MultiSourceLocation {
          start_pos: 0,
          start_depth: 0,
          end_pos: 7,
          end_depth: 1,
        },
        ..empty_block!(0)
      }
    ])
  );
  assert_doc_content!(
    resolving: b"Line-2!\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0, 39)
      }
    ])
  );
}

#[test]
fn include_with_2_trailing_newlines() {
  assert_doc_content!(
    resolving: b"Line-2!\n\n",
    adoc! {"
      Line-1
      include::some_file.adoc[]
      Line-3
    "},
    DocContent::Blocks(vecb![
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!("Line-1"; 0..6),
          node!(Inline::Newline, 6..7),
          node!("Line-2!"; 0..7, depth: 1),
        ]),
        loc: MultiSourceLocation {
          start_pos: 0,
          start_depth: 0,
          end_pos: 7,
          end_depth: 1,
        },
        ..empty_block!(0)
      },
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(just!("Line-3", 33..39)),
        ..empty_block!(33, 39)
      }
    ])
  );
}

#[test]
fn optional_include_not_found() {
  let mut parser = test_parser!("include::nope.adoc[%optional]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(ErrorResolver(ResolveError::NotFound)));
  assert!(parser.parse().is_ok());
}

assert_error!(
  include_warns_on_missing_tag,
  resolving: b"",
  adoc! {"
    include::file.adoc[tag=foo]
  "},
  error! {"
     --> test.adoc:1:24
      |
    1 | include::file.adoc[tag=foo]
      |                        ^^^ Tag `foo` not found in included file
  "}
);

assert_error!(
  include_warns_on_selecting_unclosed_tag,
  resolving: bytes! {"
    x
    // tag::unclosed[]
    a
  "},
  adoc! {"
    include::other.adoc[tag=unclosed]
  "},
  error! {"
     --> other.adoc:2:9
      |
    2 | // tag::unclosed[]
      |         ^^^^^^^^ Tag `unclosed` was not closed
  "}
);

assert_error!(
  include_warns_on_missing_tags,
  resolving: b"",
  adoc! {"
    include::file.adoc[tags=foo;bar]
  "},
  error! {"
     --> test.adoc:1:25
      |
    1 | include::file.adoc[tags=foo;bar]
      |                         ^^^^^^^ Tags `bar`, `foo` not found in included file
  "}
);

assert_no_error!(
  no_error_for_negated_missing_tag,
  resolving: b"bar",
  adoc! {"
    include::file.adoc[tag=!foo]
  "}
);

assert_error!(
  mismatched_tags,
  resolving: bytes! {"
    // tag::a[]
    a
    // tag::b[]
    b
    // end::a[]
    // end::b[]
  "},
  adoc! {"
    include::other.adoc[tags=a;b]
  "},
  error! {"
     --> other.adoc:5:9
      |
    5 | // end::a[]
      |         ^ Mismatched end tag, expected `b` but found `a`
  "}
);

assert_error!(
  unexpected_end_tag,
  resolving: bytes! {"
    // tag::a[]
    a
    // end::a[]
    // end::a[]
  "},
  adoc! {"
    include::other.adoc[tag=a]
  "},
  error! {"
     --> other.adoc:4:9
      |
    4 | // end::a[]
      |         ^ Unexpected end tag `a`
  "}
);

assert_error!(
  err_in_included_file_longer_than_parent,
  resolving: bytes! {"
    included file is longer than
    original file, to produce out of bounds
    error from incorrect diagnostic file

    xref:a.adoc#b[c]
  "},
  adoc! {"
    include::other.adoc[]
  "},
  error! {"
     --> other.adoc:5:6
      |
    5 | xref:a.adoc#b[c]
      |      ^^^^^^^^ Invalid cross reference, no anchor found for `a.adoc#b`
  "}
);

assert_error!(
  csv_unterminated_quote_err_from_include,
  resolving: b"one,\"two",
  adoc! {r#"
      ,===
      include::other.csv[]
      ,===
    "#},
  error! { r#"
       --> other.csv:1:5
        |
      1 | one,"two
        |     ^ Unclosed CSV quote
    "#}
);

#[test]
fn include_resolver_gets_passed_correct_target() {
  let cases = [
    ("include::spaced file.adoc[]", "spaced file.adoc"),
    ("include::with{sp}attr.adoc[]", "with attr.adoc"),
    (":myfile: foo.adoc\n\ninclude::{myfile}[]", "foo.adoc"),
  ];
  for (input, expected) in cases {
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::r#unsafe());
    parser.set_resolver(Box::new(AssertResolver::new(expected)));
    assert!(parser.parse().is_ok());
  }
}

#[test]
fn include_directive_resolves_attr_refs() {
  let input = adoc! {"
    :fixturesdir: fixtures/dir
    :ext: adoc

    include::{fixturesdir}/other.{ext}[]
  "};
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());

  #[cfg(unix)]
  parser.set_resolver(Box::new(AssertResolver::new("fixtures/dir/other.adoc")));

  #[cfg(windows)]
  parser.set_resolver(Box::new(AssertResolver::new("fixtures\\dir\\other.adoc")));

  assert!(parser.parse().is_ok());
}

#[test]
fn include_resolver_error_bad_encoding() {
  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let invalid_utf8 = vec![0xFF, 0xFE, 0x68, 0x00, 0xFF, 0xDC];
  parser.set_resolver(Box::new(ConstResolver(invalid_utf8)));
  let expected_err = error! {"
     --> test.adoc:1:10
      |
    1 | include::file.adoc[]
      |          ^^^^^^^^^ Error resolving file contents: Invalid UTF-16 (LE)
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);

  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let invalid_utf8 = vec![0x68, 0x00, 0xFF, 0xDC]; // <-- no BOM
  parser.set_resolver(Box::new(ConstResolver(invalid_utf8)));
  let expected_err = error! {"
     --> test.adoc:1:10
      |
    1 | include::file.adoc[]
      |          ^^^^^^^^^ Error resolving file contents: Invalid UTF-8
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);
}

#[test]
fn include_resolver_error_no_resolver() {
  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let expected_err = error! {"
     --> test.adoc:1:1
      |
    1 | include::file.adoc[]
      | ^^^^^^^^^ No resolver supplied for include directive
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);
}

#[test]
fn include_resolver_error_uri_read_not_supported() {
  let mut parser = test_parser!("include::http://a.com/b[]");
  let mut settings = JobSettings::r#unsafe();
  settings
    .job_attrs
    .insert_unchecked("allow-uri-read", JobAttr::readonly(true));
  parser.apply_job_settings(settings);
  parser.set_resolver(Box::new(ErrorResolver(ResolveError::UriReadNotSupported)));
  let expected_err = error! {"
     --> test.adoc:1:10
      |
    1 | include::http://a.com/b[]
      |          ^^^^^^^^^^^^^^ Include error: URI read not supported
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);
}

#[test]
fn uri_read_not_allowed_include() {
  // strict mode error
  let input = "include::https://my.com/foo.adoc[]";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  let expected = error! {"
     --> test.adoc:1:10
      |
    1 | include::https://my.com/foo.adoc[]
      |          ^^^^^^^^^^^^^^^^^^^^^^^ Cannot include URL contents (allow-uri-read not enabled)
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth() {
  let input = "include::file.adoc[]";
  let mut parser = test_parser!(input);
  let mut settings = JobSettings::r#unsafe();
  settings
    .job_attrs
    .insert_unchecked("max-include-depth", JobAttr::readonly("20"));
  parser.apply_job_settings(settings);
  parser.set_resolver(Box::new(InfiniteResolver::new()));
  let expected = error! {"
     --> file-20.adoc:3:1
      |
    3 | include::file-21.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 20 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth_0() {
  let input = "include::file.adoc[]";
  let mut parser = test_parser!(input);
  let mut settings = JobSettings::r#unsafe();
  settings
    .job_attrs
    .insert_unchecked("max-include-depth", JobAttr::readonly("0"));
  parser.apply_job_settings(settings);
  parser.set_resolver(Box::new(InfiniteResolver::new()));
  let expected = error! {"
     --> test.adoc:1:1
      |
    1 | include::file.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 0 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth_rel() {
  let input = "include::file.adoc[depth=10]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(InfiniteResolver::new()));
  let expected = error! {"
     --> file-11.adoc:3:1
      |
    3 | include::file-12.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 10 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth_nested_depth_1() {
  let input = "include::file.adoc[depth=1]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![
    "\ninclude::child-include.adoc[]\n",
    "\ninclude::grandchild-include.adoc[]\n",
  ])));
  let expected = error! {"
     --> child-include.adoc:2:1
      |
    2 | include::grandchild-include.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 1 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth_nested_depth_2() {
  let input = "include::file.adoc[depth=2]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![
    "include::child-include.adoc[]\n",
    "include::grandchild-include.adoc[]\n",
    "include::ggg-include.adoc[]\n",
  ])));
  let expected = error! {"
     --> grandchild-include.adoc:1:1
      |
    1 | include::ggg-include.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 2 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn max_include_depth_nested_context_exceeded() {
  let input = "include::file.adoc[depth=3]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![
    "\ninclude::child-include.adoc[depth=0]\n", // <-- included file stops
    "\ninclude::grandchild-include.adoc[]\n",
    "\ninclude::ggg-include.adoc[]\n",
  ])));
  let expected = error! {"
     --> child-include.adoc:2:1
      |
    2 | include::grandchild-include.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 0 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn include_resolution_para_newlines_edge_case() {
  let input = "include::file.adoc[depth=2]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![
    // the newlines at the beginning of these file exercizes
    // the parser's ability to correctly resolve paragraphs
    "\ninclude::child-include.adoc[]\n",
    "\ninclude::grandchild-include.adoc[]\n",
    "\ninclude::ggg-include.adoc[]\n",
  ])));
  let expected = error! {"
     --> grandchild-include.adoc:2:1
      |
    2 | include::ggg-include.adoc[]
      | ^^^^^^^^^^^^^^^^^^^^^^^^^^^ Maximum include depth of 2 exceeded
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

#[test]
fn issue_68() {
  let input = "include::include1.adoc[]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![
    "include::include2.adoc[tags=x]\n",
    "", // <-- empty file
  ])));
  _ = parser.parse(); // no panic
}

#[test]
fn issue_70_tag_removal_diagnostic_pos() {
  let input = "include::file.adoc[tag=x]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![adoc! {"
      // tag::y[]
      // end::y[]
      // tag::x[]

      [.unattached]

      // end::x[]
    "}])));
  let expected = error! {"
     --> file.adoc:5:1
      |
    5 | [.unattached]
      | ^^^^^^^^^^^^^ Unattached block metadata
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);

  let input = "include::file.adoc[lines=4..6]\n";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  parser.set_resolver(Box::new(NestedResolver(vec![adoc! {"
      line 1
      line 2
      line 3

      [.unattached]

      line 7
    "}])));
  let expected = error! {"
     --> file.adoc:5:1
      |
    5 | [.unattached]
      | ^^^^^^^^^^^^^ Unattached block metadata
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected, from: input);
}

// test resolvers

struct AssertResolver {
  expected: String,
  resolve_called: bool,
}

impl AssertResolver {
  fn new(expected: impl Into<String>) -> Self {
    Self {
      expected: expected.into(),
      resolve_called: false,
    }
  }
}

impl IncludeResolver for AssertResolver {
  fn resolve(
    &mut self,
    target: IncludeTarget,
    _: &mut dyn IncludeBuffer,
    _: SafeMode,
  ) -> std::result::Result<usize, ResolveError> {
    self.resolve_called = true;
    assert_eq!(target, IncludeTarget::FilePath(self.expected.clone()));
    Ok(0)
  }
  fn get_base_dir(&self) -> Option<String> {
    Some(String::new())
  }
  fn clone_box(&self) -> Box<dyn IncludeResolver> {
    unimplemented!()
  }
}

impl Drop for AssertResolver {
  fn drop(&mut self) {
    assert!(self.resolve_called);
  }
}

struct InfiniteResolver(usize);

impl InfiniteResolver {
  const fn new() -> Self {
    Self(0)
  }
}

impl IncludeResolver for InfiniteResolver {
  fn resolve(
    &mut self,
    _: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
    _: SafeMode,
  ) -> std::result::Result<usize, ResolveError> {
    self.0 += 1;
    let file = format!("file-{}\n\ninclude::file-{}.adoc[]\n", self.0, self.0 + 1);
    let file_bytes = file.as_bytes();
    buffer.initialize(file_bytes.len());
    let dest = buffer.as_bytes_mut();
    dest.copy_from_slice(file_bytes);
    Ok(file_bytes.len())
  }
  fn get_base_dir(&self) -> Option<String> {
    Some(String::new())
  }
  fn clone_box(&self) -> Box<dyn IncludeResolver> {
    unimplemented!()
  }
}

struct NestedResolver(Vec<&'static str>);

impl IncludeResolver for NestedResolver {
  fn resolve(
    &mut self,
    _: IncludeTarget,
    buffer: &mut dyn IncludeBuffer,
    _: SafeMode,
  ) -> std::result::Result<usize, ResolveError> {
    let file_bytes = self.0.remove(0).as_bytes();
    buffer.initialize(file_bytes.len());
    let dest = buffer.as_bytes_mut();
    dest.copy_from_slice(file_bytes);
    Ok(file_bytes.len())
  }
  fn get_base_dir(&self) -> Option<String> {
    Some(String::new())
  }
  fn clone_box(&self) -> Box<dyn IncludeResolver> {
    unimplemented!()
  }
}
