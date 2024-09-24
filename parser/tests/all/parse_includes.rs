use asciidork_ast::{prelude::*, IncludeBoundaryKind as Boundary};
use asciidork_meta::{JobAttr, JobSettings};
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 5..30),
          node!("windows"; 0..7, depth: 1), // <-- trimmed
          node!(Inline::Newline, 7..8, depth: 1),
          node!("and unix"; 8..16, depth: 1), // <-- trimmed
          node!(Inline::IncludeBoundary(Boundary::End, 1), 5..30),
        ]),
        ..empty_block!(0)
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 5..28),
          node!("text "; 0..5, depth: 1), // <-- not trimmed
          node!(Inline::IncludeBoundary(Boundary::End, 1), 5..28),
        ]),
        ..empty_block!(0)
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
        ]),
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0)
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
        ]),
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
          // node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::Newline, 7..8, depth: 1),
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(0)
      }
    ])
  );
  let input = adoc! {"
    Line-1
    include::some_file.adoc[]
    Line-3
  "};
  assert_eq!("Line-3", &input[33..39]);
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
          node!(Inline::IncludeBoundary(Boundary::Begin, 1), 7..32),
          node!("Line-2!"; 0..7, depth: 1),
        ]),
        ..empty_block!(0)
      },
      Block {
        context: BlockContext::Paragraph,
        content: BlockContent::Simple(nodes![
          node!(Inline::IncludeBoundary(Boundary::End, 1), 7..32),
          node!("Line-3"; 33..39),
        ]),
        ..empty_block!(7)
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
    1: include::file.adoc[tag=foo]
                              ^^^ Tag `foo` not found in included file
  "}
);

assert_error!(
  include_warns_on_missing_tags,
  resolving: b"",
  adoc! {"
    include::file.adoc[tags=foo;bar]
  "},
  error! {"
    1: include::file.adoc[tags=foo;bar]
                               ^^^^^^^ Tags `bar`, `foo` not found in included file
  "}
);

assert_no_error!(
  no_error_for_negated_missing_tag,
  resolving: b"bar",
  adoc! {"
    include::file.adoc[tag=!foo]
  "}
);

#[test]
fn include_resolver_gets_passed_correct_target() {
  struct AssertResolver(&'static str);
  impl IncludeResolver for AssertResolver {
    fn resolve(
      &mut self,
      target: IncludeTarget,
      _: &mut dyn IncludeBuffer,
    ) -> std::result::Result<usize, ResolveError> {
      assert_eq!(target, IncludeTarget::FilePath(self.0.to_string()));
      Ok(0)
    }
    fn get_base_dir(&self) -> Option<String> {
      Some(String::new())
    }
  }
  let cases = [
    ("include::spaced file.adoc[]", "spaced file.adoc"),
    ("include::with{sp}attr.adoc[]", "with attr.adoc"),
    (":myfile: foo.adoc\n\ninclude::{myfile}[]", "foo.adoc"),
  ];
  for (input, expected) in cases {
    let mut parser = test_parser!(input);
    parser.apply_job_settings(JobSettings::r#unsafe());
    parser.set_resolver(Box::new(AssertResolver(expected)));
    assert!(parser.parse().is_ok());
  }
}

#[test]
fn include_resolver_error_bad_encoding() {
  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let invalid_utf8 = vec![0xFF, 0xFE, 0x68, 0x00, 0xFF, 0xDC];
  parser.set_resolver(Box::new(ConstResolver(invalid_utf8)));
  let expected_err = error! {"
    1: include::file.adoc[]
                ^^^^^^^^^ Error resolving file contents: Invalid UTF-16 (LE)
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);

  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let invalid_utf8 = vec![0x68, 0x00, 0xFF, 0xDC]; // <-- no BOM
  parser.set_resolver(Box::new(ConstResolver(invalid_utf8)));
  let expected_err = error! {"
    1: include::file.adoc[]
                ^^^^^^^^^ Error resolving file contents: Invalid UTF-8
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);
}

#[test]
fn include_resolver_error_no_resolver() {
  let mut parser = test_parser!("include::file.adoc[]");
  parser.apply_job_settings(JobSettings::r#unsafe());
  let expected_err = error! {"
    1: include::file.adoc[]
       ^^^^^^^^^ No resolver supplied for include directive
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
    1: include::http://a.com/b[]
                ^^^^^^^^^^^^^^ Include resolver error: URI read not supported
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err);
}

#[test]
fn uri_read_not_allowed_include() {
  // strict mode error
  let input = "include::https://my.com/foo.adoc[]";
  let mut parser = test_parser!(input);
  parser.apply_job_settings(JobSettings::r#unsafe());
  let expected_err = error! {"
    1: include::https://my.com/foo.adoc[]
                ^^^^^^^^^^^^^^^^^^^^^^^ Cannot include URL contents (allow-uri-read not enabled)
  "};
  expect_eq!(parser.parse().err().unwrap()[0].plain_text(), expected_err, from: input);
}

// attrs, encodings, etc
// include on last line of para
// consecutive includes to define attrs
// error messages
