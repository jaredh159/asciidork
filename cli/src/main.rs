use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::result::Result;
use std::time::{Duration, Instant};
use std::{error::Error, fs};

use bumpalo::Bump;
use clap::Parser as ClapParser;
use colored::*;

use asciidork_dr_html_backend::*;
use asciidork_parser::parser::Parser;
use asciidork_parser::Diagnostic;

mod args;

use args::{Args, Output};

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  let src = {
    if let Some(file) = &args.input {
      let mut file = fs::File::open(file)?;
      let mut src = file
        .metadata()
        .ok()
        .map(|metadata| String::with_capacity(metadata.len() as usize))
        .unwrap_or_else(String::new);
      file.read_to_string(&mut src)?;
      src
    } else {
      let mut src = String::new();
      std::io::stdin().read_to_string(&mut src)?;
      src
    }
  };

  let parse_start = Instant::now();
  let bump = &Bump::with_capacity(src.len());
  let parser = Parser::new(bump, &src);
  let result = parser.parse();
  let parse_time = parse_start.elapsed();

  match result {
    Ok(parse_result) => match &args.format {
      Output::DrHtml | Output::DrHtmlPrettier => {
        let convert_start = Instant::now();
        let mut html = convert(parse_result.document, args.clone().into())?;
        let convert_time = convert_start.elapsed();
        let prettify = args.format == Output::DrHtmlPrettier;
        if prettify {
          html = format_html(html);
        }
        if let Some(file) = &args.output {
          fs::write(file, html)?;
        } else {
          eprintln_if(prettify);
          println!("{html}");
        }
        if args.print_timings {
          eprintln_if(!prettify);
          print_timings(src.len(), parse_time, Some(convert_time));
        }
      }
      Output::Ast => {
        println!("\n{:#?}", parse_result.document);
        if args.print_timings {
          eprintln!();
          print_timings(src.len(), parse_time, None);
        }
      }
    },
    Err(diagnostics) => {
      print_diagnostics(diagnostics);
    }
  }
  Ok(())
}

fn print_timings(len: usize, parse_time: Duration, convert_time: Option<Duration>) {
  if cfg!(debug_assertions) {
    eprintln!(
      " {} {}\n",
      "WARN:".red().bold(),
      "This is a debug build, will be MUCH slower than a release build!"
        .white()
        .dimmed()
    );
  }
  eprintln!(
    " {} {} {}",
    "Input size:   ".white().dimmed(),
    format!("{:.2?}", len).green().bold(),
    "bytes".white().dimmed()
  );
  eprintln!(
    " {} {}",
    "Parse time:   ".white().dimmed(),
    format!("{:.2?}", parse_time).green().bold()
  );
  if let Some(convert_time) = convert_time {
    eprintln!(
      " {} {}",
      "Convert time: ".white().dimmed(),
      format!("{:.2?}", convert_time).green().bold()
    );
    eprintln!(
      " {} {}",
      "Total time:   ".white().dimmed(),
      format!("{:.2?}", parse_time + convert_time,).green().bold()
    );
  }
}

fn format_html(html: String) -> String {
  let mut child = Command::new("prettier")
    .arg("--parser")
    .arg("html")
    .arg("--html-whitespace-sensitivity")
    .arg("ignore")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("`--format dr-html-pretty` requires `prettier` to be installed.\n");
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

fn eprintln_if(condition: bool) {
  if condition {
    eprintln!();
  }
}
