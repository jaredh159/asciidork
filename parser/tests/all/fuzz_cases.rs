use asciidork_parser::prelude::*;

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
