#![no_main]

use asciidork_parser::prelude::*;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &str| {
  let bump = &bumpalo::Bump::new();
  let parser = Parser::from_str(input, SourceFile::Tmp, bump);
  let _ = parser.parse();
});
