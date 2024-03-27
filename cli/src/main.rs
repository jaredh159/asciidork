use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::result::Result;
use std::{env, error::Error, fs};

use asciidork_dr_html_backend::AsciidoctorHtml;
use asciidork_eval::{eval, Opts};
use asciidork_parser::parser::Parser;
use asciidork_parser::Diagnostic;
use bumpalo::Bump;
use colored::*;

fn main() -> Result<(), Box<dyn Error>> {
  let args = env::args().skip(1).collect::<Vec<String>>();
  let mut file = fs::File::open(&args[1])?;
  let mut src = String::new();
  file.read_to_string(&mut src).unwrap();
  let bump = &Bump::with_capacity(src.len());
  let parser = Parser::new(bump, &src);
  let result = parser.parse();
  match result {
    Ok(parse_result) => match args[0].as_str() {
      "print-ast" => println!("{:#?}", parse_result.document),
      "print-html" => {
        let html = eval(
          parse_result.document,
          Opts::default(),
          AsciidoctorHtml::new(),
        )?;
        println!("\n{}", format_html(html));
      }
      _ => panic!("Unknown command"),
    },
    Err(diagnostics) => {
      print_diagnostics(diagnostics);
    }
  }
  Ok(())
}

fn format_html(html: String) -> String {
  let mut child = Command::new("prettier")
    .arg("--parser")
    .arg("html")
    .arg("--print-width")
    .arg("60")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .unwrap();
  let child_stdin = child.stdin.as_mut().unwrap();
  child_stdin.write_all(html.as_bytes()).unwrap();
  let output = child.wait_with_output().unwrap();
  String::from_utf8_lossy(&output.stdout).to_string()
}

fn print_diagnostics(diagnostics: Vec<Diagnostic>) {
  for diagnostic in diagnostics {
    let line_num_pad = match diagnostic.line_num {
      n if n < 10 => 3,
      n if n < 100 => 4,
      n if n < 1000 => 5,
      n if n < 10000 => 6,
      _ => 7,
    };
    println!(
      "\n{}{} {}",
      diagnostic.line_num.to_string().dimmed(),
      ":".dimmed(),
      diagnostic.line
    );
    println!(
      "{}{} {}\n",
      " ".repeat(diagnostic.underline_start + line_num_pad),
      "^".repeat(diagnostic.underline_width).red().bold(),
      diagnostic.message.red().bold()
    );
  }
}
