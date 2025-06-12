use std::env;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::result::Result;
use std::time::{Duration, Instant, SystemTime};
use std::{error::Error, fs, time::UNIX_EPOCH};

use bumpalo::Bump;
use clap::Parser as ClapParser;
use colored::*;

use asciidork_core::{JobSettings, Path};
use asciidork_dr_html_backend::*;
use asciidork_parser::prelude::*;

mod args;
mod css;
mod error;
mod resolver;

use args::{Args, Output};
use error::DiagnosticError;
use resolver::CliResolver;

fn main() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  run(args, std::io::stdin(), std::io::stdout(), std::io::stderr())
}

fn run(
  args: Args,
  mut stdin: impl Read,
  mut stdout: impl Write,
  mut stderr: impl Write,
) -> Result<(), Box<dyn Error>> {
  let (src, src_file, base_dir, input_mtime) = {
    if let Some(pathbuf) = &args.input {
      let abspath = dunce::canonicalize(pathbuf)?;
      let mut file = fs::File::open(pathbuf.clone())?;
      let mut input_mtime = None;
      let mut src = file
        .metadata()
        .ok()
        .map(|metadata| {
          if let Ok(mtime) = metadata.modified() {
            input_mtime = Some(mtime.duration_since(UNIX_EPOCH).unwrap().as_secs());
          }
          String::with_capacity(metadata.len() as usize)
        })
        .unwrap_or_else(String::new);
      // TODO: for perf, better to read the file straight into a BumpVec<u8>
      // have an initializer on Parser that takes ownership of it
      file.read_to_string(&mut src)?;
      let base_dir = args
        .base_dir
        .as_ref()
        .cloned()
        .or_else(|| abspath.parent().map(|p| p.to_path_buf()));
      (src, SourceFile::Path(abspath.into()), base_dir, input_mtime)
    } else {
      let mut src = String::new();
      stdin.read_to_string(&mut src)?;
      let cwd_buf = env::current_dir()?;
      let cwd = Path::new(cwd_buf.to_str().unwrap_or(""));
      (src, SourceFile::Stdin { cwd }, Some(cwd_buf), None)
    }
  };

  let parse_start = Instant::now();
  let bump = &Bump::with_capacity(src.len() * 2);
  let strict = args.strict;
  let mut parser = Parser::from_str(&src, src_file, bump);
  let mut job_settings: JobSettings = args.clone().try_into()?;
  AsciidoctorHtml::set_job_attrs(&mut job_settings.job_attrs);
  parser.apply_job_settings(job_settings);
  parser.set_resolver(Box::new(CliResolver::new(base_dir, strict)));

  let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
  parser.provide_timestamps(now, input_mtime, None);

  let result = parser.parse();
  let parse_time = parse_start.elapsed();

  match result {
    Ok(mut parse_result) => match &args.format {
      Output::DrHtml | Output::DrHtmlPrettier => {
        let convert_start = Instant::now();
        if let Err(css_err) = css::resolve(&mut parse_result.document) {
          writeln!(stderr, "ERROR: {}", css_err)?;
          if args.strict {
            std::process::exit(1);
          }
        }
        let mut html = convert(parse_result.document)?;
        let convert_time = convert_start.elapsed();
        let prettify = args.format == Output::DrHtmlPrettier;
        if prettify {
          html = format_html(html);
        }
        if let Some(file) = &args.output {
          fs::write(file, html)?;
        } else {
          if prettify {
            writeln!(stderr)?;
          }
          writeln!(stdout, "{html}")?;
        }
        if args.print_timings {
          if !prettify {
            writeln!(stderr)?;
          }
          print_timings(&mut stderr, src.len(), parse_time, Some(convert_time));
        }
      }
    },
    Err(diagnostics) => {
      if args.json_errors {
        print_json_diagnostics(&mut stderr, diagnostics);
        std::process::exit(1);
      } else {
        print_human_diagnostics(&mut stderr, diagnostics);
        return Err("Parse error".into());
      }
    }
  }
  Ok(())
}

fn print_timings(
  dest: &mut impl Write,
  len: usize,
  parse_time: Duration,
  convert_time: Option<Duration>,
) {
  if cfg!(debug_assertions) {
    writeln!(
      dest,
      " {} {}\n",
      "WARN:".red().bold(),
      "This is a debug build, will be MUCH slower than a release build!"
        .white()
        .dimmed()
    )
    .unwrap();
  }
  writeln!(
    dest,
    " {} {} {}",
    "Input size:   ".white().dimmed(),
    format!("{:.2?}", len).green().bold(),
    "bytes".white().dimmed()
  )
  .unwrap();
  writeln!(
    dest,
    " {} {}",
    "Parse time:   ".white().dimmed(),
    format!("{:.2?}", parse_time).green().bold()
  )
  .unwrap();
  if let Some(convert_time) = convert_time {
    writeln!(
      dest,
      " {} {}",
      "Convert time: ".white().dimmed(),
      format!("{:.2?}", convert_time).green().bold()
    )
    .unwrap();
    writeln!(
      dest,
      " {} {}",
      "Total time:   ".white().dimmed(),
      format!("{:.2?}", parse_time + convert_time,).green().bold()
    )
    .unwrap();
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

fn print_human_diagnostics(dest: &mut impl Write, diagnostics: Vec<Diagnostic>) {
  for diagnostic in diagnostics {
    writeln!(dest, "\n{}", diagnostic.plain_text_with(Colorizer)).unwrap();
  }
}

fn print_json_diagnostics(dest: &mut impl Write, diagnostics: Vec<Diagnostic>) {
  let errors: Vec<DiagnosticError> = diagnostics.into_iter().map(DiagnosticError::from).collect();
  let json = miniserde::json::to_string(&errors);
  writeln!(dest, "{}", json).unwrap();
}

struct Colorizer;

impl DiagnosticColor for Colorizer {
  fn line_num(&self, s: impl Into<String>) -> String {
    format!("{}", Into::<String>::into(s).dimmed())
  }
  fn location(&self, s: impl Into<String>) -> String {
    format!("{}", Into::<String>::into(s).red().bold())
  }
  fn message(&self, s: impl Into<String>) -> String {
    format!("{}", Into::<String>::into(s).red().bold())
  }
}

// hack: force cli version publish - a4f89239
