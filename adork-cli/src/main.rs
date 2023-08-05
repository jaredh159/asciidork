use std::error::Error;
use std::io;
use std::result::Result;
use std::{env, fmt, fs};

use colored::Colorize;

use adork::err::{AsciiDorkError, ParseErr};
use adork::parse::Parser;

#[derive(Debug)]
enum CliErr {
  Io(io::Error),
  Adork(AsciiDorkError),
}

fn main() {
  let args = env::args().skip(1).collect::<Vec<String>>();

  // todo, subcommands, probably a crate, etc. etc...
  match print_ast(&args[1]) {
    Err(CliErr::Adork(AsciiDorkError::Parse(parse_err))) => println!("{}", parse_err),
    Err(err) => println!("{}", err),
    Ok(_) => (),
  }
}

// adork print-ast [file]
fn print_ast(path: &str) -> Result<(), CliErr> {
  let file = fs::File::open(path)?;
  let parser = Parser::from_file(file, Some(path));
  match parser.parse() {
    Err(diagnostics) => {
      for diagnostic in diagnostics {
        println!(
          "\n{}{} {}",
          diagnostic.line_num.to_string().dimmed(),
          ":".dimmed(),
          diagnostic.line
        );
        println!(
          "{}{} {}\n",
          " ".repeat(diagnostic.message_offset),
          "^".red().bold(),
          diagnostic.message.red().bold()
        );
      }
      Ok(())
    }
    Ok(parse_result) => {
      println!("{:#?}", parse_result.document);
      Ok(())
    }
  }
}

impl fmt::Display for CliErr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CliErr::Io(err) => write!(f, "{}", err),
      CliErr::Adork(err) => write!(f, "{}", err),
    }
  }
}

impl Error for CliErr {}

impl From<io::Error> for CliErr {
  fn from(err: io::Error) -> Self {
    CliErr::Io(err)
  }
}

impl From<AsciiDorkError> for CliErr {
  fn from(err: AsciiDorkError) -> Self {
    CliErr::Adork(err)
  }
}

impl From<ParseErr> for CliErr {
  fn from(err: ParseErr) -> Self {
    CliErr::Adork(AsciiDorkError::Parse(err))
  }
}
