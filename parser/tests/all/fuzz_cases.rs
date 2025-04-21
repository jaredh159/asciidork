use asciidork_parser::prelude::*;

#[test]
fn fuzzer_inputs() {
  let cases: &[&str] = &[
    "1. ",
    "##++\r###+\t",
    "<.>",
    "&\t\r//",
    "[#s&p*#]",
    "[#]",
    "`+`<",
    "++\n++",
    "[.\0.]",
    "**\u{b})",
    "`+&+`",
    "`++\u{4}+`X+`+",
    "`++b+`X+`+",
  ];
  let bump = &bumpalo::Bump::new();
  for input in cases {
    let parser = Parser::from_str(input, SourceFile::Tmp, bump);
    let _ = parser.parse();
  }
}
