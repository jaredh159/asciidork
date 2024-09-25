use crate::internal::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Diagnostic {
  pub line_num: u32,
  pub line: String,
  pub message: String,
  pub underline_start: u32,
  pub underline_width: u32,
  pub source_file: SourceFile,
}

impl<'arena> Parser<'arena> {
  pub(crate) fn err_at(&self, message: impl Into<String>, start: u32, end: u32) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(start).to_string(),
      message: message.into(),
      underline_start: offset,
      underline_width: end - start,
      source_file: self.lexer.source_file().clone(),
    })
  }

  pub(crate) fn err_line(&self, message: impl Into<String>, line: &Line) -> Result<()> {
    let start = line
      .loc()
      .expect("non empty line for `err_entire_line`")
      .start;
    let (line_num, offset) = self.lexer.line_number_with_offset(start);
    let line = line.reassemble_src().to_string();
    self.handle_err(Diagnostic {
      line_num,
      message: message.into(),
      underline_start: offset,
      underline_width: line.len() as u32,
      line,
      source_file: self.lexer.source_file().clone(),
    })
  }

  pub(crate) fn err_doc_attr(
    &self,
    key: impl Into<String>,
    message: impl Into<String>,
  ) -> Result<()> {
    let key = &key.into();
    for (idx, line) in self
      .lexer
      .raw_lines()
      .enumerate()
      .skip_while(|(_, l)| l.is_empty())
    {
      if line.is_empty() {
        break; // must have left doc header
      }
      if line.starts_with(key) {
        return self.handle_err(Diagnostic {
          line_num: idx as u32 + 1,
          line: line.to_string(),
          message: message.into(),
          underline_start: 0,
          underline_width: line.len() as u32,
          source_file: self.lexer.source_file().clone(),
        });
      }
    }
    debug_assert!(false, "doc attr not found");
    Ok(())
  }

  pub(crate) fn err_at_pattern(
    &self,
    message: impl Into<String>,
    line_start: u32,
    pattern: &str,
  ) -> Result<()> {
    let (line_num, _) = self.lexer.line_number_with_offset(line_start);
    let line = self.lexer.line_of(line_start);
    if let Some(idx) = line.find(pattern) {
      return self.handle_err(Diagnostic {
        line_num,
        line: line.to_string(),
        message: message.into(),
        underline_start: idx as u32,
        underline_width: pattern.len() as u32,
        source_file: self.lexer.source_file().clone(),
      });
    }
    self.handle_err(Diagnostic {
      line_num,
      line: line.to_string(),
      message: message.into(),
      underline_start: 0,
      underline_width: line.len() as u32,
      source_file: self.lexer.source_file().clone(),
    })
  }

  pub(crate) fn err_at_loc(&self, message: impl Into<String>, loc: SourceLocation) -> Result<()> {
    self.err_at(message, loc.start, loc.end)
  }

  pub(crate) fn err_token_full(&self, message: impl Into<String>, token: &Token) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(token.loc.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(token.loc.start).to_string(),
      message: message.into(),
      underline_start: offset,
      underline_width: token.lexeme.len() as u32,
      source_file: self.lexer.source_file().clone(),
    })
  }

  pub(crate) fn err_token_start(&self, message: impl Into<String>, token: &Token) -> Result<()> {
    let (line_num, offset) = self.lexer.line_number_with_offset(token.loc.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(token.loc.start).to_string(),
      message: message.into(),
      underline_start: offset,
      underline_width: 1,
      source_file: self.lexer.source_file().clone(),
    })
  }

  pub(crate) fn err(&self, message: impl Into<String>, token: Option<&Token>) -> Result<()> {
    let location = token.map_or_else(|| self.lexer.loc(), |t| t.loc);
    let (line_num, offset) = self.lexer.line_number_with_offset(location.start);
    self.handle_err(Diagnostic {
      line_num,
      line: self.lexer.line_of(location.start).to_string(),
      message: message.into(),
      underline_start: offset,
      underline_width: 1,
      source_file: self.lexer.source_file().clone(),
    })
  }

  fn handle_err(&self, err: Diagnostic) -> Result<()> {
    if self.strict {
      Err(err)
    } else {
      self.errors.borrow_mut().push(err);
      Ok(())
    }
  }
}

pub trait DiagnosticColor {
  fn line_num(&self, s: impl Into<String>) -> String {
    s.into()
  }
  fn line(&self, s: impl Into<String>) -> String {
    s.into()
  }
  fn location(&self, s: impl Into<String>) -> String {
    s.into()
  }
  fn message(&self, s: impl Into<String>) -> String {
    s.into()
  }
}

impl Diagnostic {
  pub fn plain_text(&self) -> String {
    struct NoColor;
    impl DiagnosticColor for NoColor {}
    self.plain_text_with(NoColor)
  }

  pub fn plain_text_with<C: DiagnosticColor>(&self, colorizer: C) -> String {
    let line_num_pad = match self.line_num {
      n if n < 10 => 4,
      n if n < 100 => 5,
      n if n < 1000 => 6,
      n if n < 10000 => 7,
      _ => 8,
    };
    format!(
      "{}{}{}{}{}{}{}\n{}{}\n{} {} {}\n{}{} {}{} {}\n",
      " ".repeat((line_num_pad - 3) as usize),
      colorizer.line_num("--> "),
      colorizer.line_num(self.source_file.file_name()),
      colorizer.line_num(":"),
      colorizer.line_num(self.line_num.to_string()),
      colorizer.line_num(":"),
      colorizer.line_num((self.underline_start + 1).to_string()),
      " ".repeat((line_num_pad - 2) as usize),
      colorizer.line_num("|"),
      colorizer.line_num(self.line_num.to_string()),
      colorizer.line_num("|"),
      colorizer.line(&self.line),
      " ".repeat((line_num_pad - 2) as usize),
      colorizer.line_num("|"),
      " ".repeat(self.underline_start as usize),
      colorizer.location("^".repeat(self.underline_width as usize)),
      colorizer.message(&self.message),
    )
  }
}
