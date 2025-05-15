#![no_main]

use asciidork_parser::prelude::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
  let bump = &bumpalo::Bump::new();
  let mut settings = ::asciidork_core::JobSettings::embedded();
  settings.strict = false; // <--
  let mut parser = Parser::from_str(input, SourceFile::Tmp, bump);
  parser.apply_job_settings(settings);
  let _ = parser.parse();
});
