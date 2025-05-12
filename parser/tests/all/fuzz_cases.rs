use asciidork_core::SafeMode;
use asciidork_parser::prelude::*;
use test_utils::const_resolver;

#[test]
fn fuzzer_inputs() {
  let cases: &[&str] = &[
    "1. ",
    "##++\r###+\t",
    "<.>",
    "&\t\r//",
    "`+`<",
    "++\n++",
    "[.\0.]",
    "**\u{b})",
    "`+&+`",
    "`++\u{4}+`X+`+",
    "`++b+`X+`+",
    "-\n////",
    " <484>",
    "88888.\u{b}",
    "++++\n+",
    "====\n= x",
    "toc:[]",
    "#kbd:([]",
    "\n++++\n+\n++++\n\u{8}+\n\n+",
    "++++\n[]",
    ";;;\n++++\n;:: ",
    "\n++++\n$++\n++++\n++\n\nR",
    "**\u{c},",
    "-::::\t.\u{1}\n.::\t\n+\n\t",
    "=\t\0\nf\u{b}\u{b}f",
    ":notitle: empty",
    ":!chapter-refsig:",
    "[#s&p*#]",
    "[#]",
    "[[,]]",
    "[',:',]",
    "[[,(]]",
    "[[\0,]]",
    "[]]#",
    "[\\]#",
    "[\t]]#",
    "['']#",
    "[[=]#",
    "[x=]#",
    "[.]#=,_",
    "x[[x,]0]]",
    "\n////\n\n////\n\0",
    "'\n////\n/\n\n////\n",
    "x\n////\n////x",
    "\u{18}/\n////\n'\n\n",
    "\u{b}\u{b}::",
    "O::\n+\n--\n+",
    "\"\n-- e",
    "**\u{a0}\0",
    "`::\t.x\n=== d",
    "\n!===\n\n^ۂ=\r!===\n",
    ". x\n= ;;",
  ];
  let bump = &bumpalo::Bump::new();
  let last = cases[cases.len() - 1];
  let parser = Parser::from_str(last, SourceFile::Tmp, bump);
  let _ = parser.parse();
  for input in cases {
    let parser = Parser::from_str(input, SourceFile::Tmp, bump);
    let _ = parser.parse();
  }
}

#[test]
fn fuzzer_inputs_not_strict() {
  let cases: &[&str] = &[
    "[##-[#-]",
    ".\t.\n.::",
    "|===\n|",
    "[id=]",
    "|===\na|\u{1a}\n////\n",
  ];
  let bump = &bumpalo::Bump::new();
  let last = cases[cases.len() - 1];
  let mut settings = ::asciidork_core::JobSettings::embedded();
  settings.strict = false;
  let mut parser = Parser::from_str(last, SourceFile::Tmp, bump);
  parser.apply_job_settings(settings.clone());
  let _result = parser.parse();
  // assert!(_result.is_ok(), "{:?}", _result);
  for input in cases {
    let mut parser = Parser::from_str(input, SourceFile::Tmp, bump);
    parser.apply_job_settings(settings.clone());
    let _ = parser.parse();
  }
}

#[test]
fn fuzzer_includes() {
  let cases: &[(&str, &str, &str, SafeMode, bool)] = &[
    (";pass:\n", "", "", SafeMode::Secure, false),
    ("x\n", "////\nx", "////\n", SafeMode::Server, false),
    // NB: below was found, but not fixed
    // (
    //   "",
    //   "make-long-enough\n\n|===\na|\n--\n",
    //   "x",
    //   SafeMode::Safe,
    //   false,
    // ),
  ];
  let bump = &bumpalo::Bump::new();
  for (before, included, after, safe, strict) in cases {
    let mut input = String::from(*before);
    if !input.ends_with('\n') {
      input.push('\n');
    }
    input.push_str("include::file.adoc[]\n");
    input.push_str(after);
    let mut parser = Parser::from_str(&input, SourceFile::Tmp, bump);
    let mut settings = ::asciidork_core::JobSettings::embedded();
    settings.safe_mode = *safe;
    settings.strict = *strict;
    parser.apply_job_settings(settings);
    parser.set_resolver(const_resolver!(included.as_bytes()));
    let _ = parser.parse();
  }
}
