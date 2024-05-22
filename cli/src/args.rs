use asciidork_meta::DocType;
use asciidork_opts::Opts;
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
  pub doc_type: DocType,

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

impl From<Args> for Opts {
  fn from(args: Args) -> Self {
    Opts {
      // TODO: this might not be correct...
      doc_type: if args.embedded { DocType::Inline } else { args.doc_type },
      ..Opts::default()
    }
  }
}
