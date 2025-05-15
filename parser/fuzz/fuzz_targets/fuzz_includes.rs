#![no_main]

use asciidork_core::SafeMode;
use asciidork_parser::includes::ConstResolver;
use asciidork_parser::prelude::*;
use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;

#[derive(Clone, Debug)]
struct Inputs<'a> {
  before_include: &'a str,
  included: &'a str,
  after_include: &'a str,
  safemode: SafeMode,
  strict: bool,
}

impl<'a> Arbitrary<'a> for Inputs<'a> {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    let b1: bool = u.arbitrary()?;
    let b2: bool = u.arbitrary()?;
    let safemode = match (b1, b2) {
      (true, true) => SafeMode::Unsafe,
      (true, false) => SafeMode::Safe,
      (false, true) => SafeMode::Server,
      (false, false) => SafeMode::Secure,
    };
    Ok(Inputs {
      before_include: u.arbitrary()?,
      included: u.arbitrary()?,
      after_include: u.arbitrary()?,
      safemode,
      strict: u.arbitrary()?,
    })
  }
}

fuzz_target!(|inputs: Inputs| {
  let bump = &bumpalo::Bump::new();
  let mut input = String::from(inputs.before_include);
  if !input.ends_with('\n') {
    input.push('\n');
  }
  input.push_str("include::file.adoc[]\n");
  input.push_str(inputs.after_include);
  let mut settings = ::asciidork_core::JobSettings::embedded();
  settings.safe_mode = inputs.safemode;
  settings.strict = inputs.strict;
  let mut parser = Parser::from_str(&input, SourceFile::Tmp, bump);
  parser.apply_job_settings(settings);
  let resolver = Box::new(ConstResolver(Vec::from(inputs.included.as_bytes())));
  parser.set_resolver(resolver);
  let _ = parser.parse();
});
