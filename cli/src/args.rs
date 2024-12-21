use asciidork_meta::{DocType, JobAttr, JobAttrs, JobSettings, SafeMode};
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

#[derive(Parser, Debug, Clone)]
#[command(version, about = "🤓 Asciidork CLI")]
#[command(name = "asciidork", bin_name = "asciidork")]
pub struct Args {
  #[clap(short, long, help = "The file path to parse - omit to read from stdin")]
  pub input: Option<std::path::PathBuf>,

  #[clap(short, long, default_value = "dr-html")]
  #[clap(help = "Select output format")]
  pub format: Output,

  #[arg(value_parser = DocType::from_str)]
  #[clap(short, long, default_value = "article")]
  #[clap(help = "Document type to use when converting")]
  pub doctype: DocType,

  #[arg(value_parser = parse_attr)]
  #[clap(short, long = "attribute")]
  #[clap(
    help = "Set a document attribute (i.e., name=value, name@=value, name!, name) - may be set more than once"
  )]
  pub attributes: Vec<(String, JobAttr)>,

  #[arg(value_parser = SafeMode::from_str)]
  #[clap(short, long, default_value = "secure")]
  #[clap(help = "Set safe mode explicitly")]
  pub safe_mode: SafeMode,

  #[clap(short, long, help = "Output file path - omit to write to stdout")]
  pub output: Option<std::path::PathBuf>,

  #[clap(short, long, default_value = "false")]
  #[clap(help = "Supress enclosing document structure")]
  pub embedded: bool,

  #[clap(long, default_value = "false")]
  pub strict: bool,

  #[clap(
    short = 'B',
    long,
    help = "Base directory for includes, resources (default: directory of entry file)"
  )]
  pub base_dir: Option<std::path::PathBuf>,

  #[clap(short = 't', long, default_value = "false")]
  #[clap(help = "Print timing/perf info\n")]
  pub print_timings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Output {
  DrHtml,
  DrHtmlPrettier,
}

lazy_static! {
  pub static ref ATTR_RE: Regex = Regex::new(r"(\w(?:[\w-]*))(!)?(@)?(?:=(.+))?").unwrap();
}

fn parse_attr(input: &str) -> Result<(String, JobAttr), &'static str> {
  let captures = ATTR_RE.captures(input).ok_or("Invalid attribute")?;
  let key = captures.get(1).unwrap().as_str().to_lowercase().to_string();
  match (
    captures.get(2).is_some(),
    captures.get(3).is_some(),
    captures.get(4).map(|m| m.as_str()),
  ) {
    // ! ,   @
    (false, false, None) => Ok((key, JobAttr::readonly(true))),
    (false, true, None) => Ok((key, JobAttr::modifiable(true))),
    (true, false, None) => Ok((key, JobAttr::readonly(false))),
    (true, true, None) => Ok((key, JobAttr::modifiable(false))),
    (false, true, Some(value)) => Ok((key, JobAttr::modifiable(value))),
    (false, false, Some(value)) if value.ends_with('@') => {
      Ok((key, JobAttr::modifiable(value.trim_end_matches('@'))))
    }
    (false, false, Some(value)) => Ok((key, JobAttr::readonly(value))),
    (true, _, Some(_)) => Err("Cannot unset attr with `!` AND provide value"),
  }
}

#[test]
fn test_parse_job_attr() {
  let cases = [
    ("foo=bar", ("foo", JobAttr::readonly("bar"))),
    ("FOO=bar", ("foo", JobAttr::readonly("bar"))),
    ("foo=bar@", ("foo", JobAttr::modifiable("bar"))),
    ("foo@=bar", ("foo", JobAttr::modifiable("bar"))),
    ("foo", ("foo", JobAttr::readonly(true))),
    ("foo@", ("foo", JobAttr::modifiable(true))),
    ("foo!", ("foo", JobAttr::readonly(false))),
    ("foo!@", ("foo", JobAttr::modifiable(false))),
    ("foo=bar baz", ("foo", JobAttr::readonly("bar baz"))),
  ];

  for (input, (key, job_attr)) in cases.iter() {
    assert_eq!(
      parse_attr(input).unwrap(),
      (key.to_string(), job_attr.clone())
    );
  }
}

impl TryFrom<Args> for JobSettings {
  type Error = String;
  fn try_from(args: Args) -> Result<Self, Self::Error> {
    let mut j = JobSettings {
      safe_mode: args.safe_mode,
      doctype: Some(args.doctype),
      embedded: args.embedded,
      strict: args.strict,
      job_attrs: JobAttrs::empty(),
    };
    for (key, attr) in args.attributes {
      j.job_attrs.insert(key, attr)?;
    }
    Ok(j)
  }
}
