use std::io::Read;
use std::result::Result;
use std::{env, fs};

use bumpalo::Bump;
use colored::*;

use asciidork_parser::parser::Parser;

fn main() {
  let args = env::args().skip(1).collect::<Vec<String>>();
  let res = print_ast(&args[1]);
  println!("{:?}", res);
}

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

// adork print-ast [file]
fn print_ast(path: &str) -> Result<(), GenericError> {
  let mut file = fs::File::open(path)?;
  let mut src = String::new();
  file.read_to_string(&mut src)?;
  let bump = &Bump::with_capacity(src.len());
  let parser = Parser::new(bump, &src);
  let result = parser.parse();
  match result {
    Err(diagnostics) => {
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
      Ok(())
    }
    Ok(parse_result) => {
      println!("{:#?}", parse_result.document);
      Ok(())
    }
  }
  // println!("{:#?}", node);
  // Ok(())
}

// this shows how to read the file right into a bump vec
// fn print_ast(path: &str) -> Result<(), GenericError> {
//   let file_size = file.metadata()?.len() as usize;
//   let file = fs::File::open(path)?;
//   let bump = Bump::with_capacity(file_size);
//   let mut buf: BumpVec<u8> = BumpVec::with_capacity_in(file_size, &bump);
//   buf.resize(file_size, 0);
//   let mut reader = io::BufReader::new(file);
//   loop {
//     if let 0 = reader.read(&mut buf)? {
//       break;
//     }
//   }
//   let src = std::str::from_utf8(&buf)?;
//   let mut parser = Parser::new(&bump, src);
//   let node = parser.parse();
//   println!("{:#?}", node);
//   Ok(())
// }

// #[derive(Debug)]
// enum CliErr {
//   Io(io::Error),
//   Adork(AsciiDorkError),
// }

// let parser = Parser::from_file(file, Some(path));
// match parser.parse() {
//   Err(diagnostics) => {
//     for diagnostic in diagnostics {
//       let line_num_pad = match diagnostic.line_num {
//         n if n < 10 => 3,
//         n if n < 100 => 4,
//         n if n < 1000 => 5,
//         n if n < 10000 => 6,
//         _ => 7,
//       };
//       println!(
//         "\n{}{} {}",
//         diagnostic.line_num.to_string().dimmed(),
//         ":".dimmed(),
//         diagnostic.line
//       );
//       println!(
//         "{}{} {}\n",
//         " ".repeat(diagnostic.underline_start + line_num_pad),
//         "^".repeat(diagnostic.underline_width).red().bold(),
//         diagnostic.message.red().bold()
//       );
//     }
//     Ok(())
//   }
//   Ok(parse_result) => {
//     println!("{:#?}", parse_result.document);
//     Ok(())
//   }
// }
// }

// impl fmt::Display for CliErr {
//   fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//     match self {
//       CliErr::Io(err) => write!(f, "{}", err),
//       CliErr::Adork(err) => write!(f, "{}", err),
//     }
//   }
// }

// impl Error for CliErr {}

// impl From<io::Error> for CliErr {
//   fn from(err: io::Error) -> Self {
//     CliErr::Io(err)
//   }
// }

// impl From<AsciiDorkError> for CliErr {
//   fn from(err: AsciiDorkError) -> Self {
//     CliErr::Adork(err)
//   }
// }

// impl From<ParseErr> for CliErr {
//   fn from(err: ParseErr) -> Self {
//     CliErr::Adork(AsciiDorkError::Parse(err))
//   }
// }
