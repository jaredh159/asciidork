use asciidork_meta::{DocType, JobSettings, SafeMode};
use clap::Parser;
use std::str::FromStr;

// TODO: add  `-a, --attribute name[=value]`
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

  #[arg(value_parser = SafeMode::from_str)]
  #[clap(short, long, default_value = "secure")]
  #[clap(help = "Set safe mode explicitly")]
  pub safe_mode: SafeMode,

  #[clap(short, long, help = "Output file path - omit to write to stdout")]
  pub output: Option<std::path::PathBuf>,

  #[clap(short, long, default_value = "false")]
  #[clap(help = "Supress enclosing document structure")]
  pub embedded: bool,

  #[clap(short = 't', long, default_value = "false")]
  #[clap(help = "Print timing/perf info\n")]
  pub print_timings: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Output {
  DrHtml,
  DrHtmlPrettier,
  Ast,
}

impl From<Args> for JobSettings {
  fn from(args: Args) -> Self {
    let mut settings = JobSettings {
      safe_mode: args.safe_mode,
      doctype: Some(args.doctype),
      embedded: args.embedded,
      strict: true,
      ..JobSettings::default()
    };
    settings
      .job_attrs
      .insert_unchecked("allow-uri-read", asciidork_meta::JobAttr::readonly(true));
    settings
  }
}
