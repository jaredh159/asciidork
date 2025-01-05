use lazy_static::lazy_static;
use regex::Regex;
use Inline::Symbol;

use crate::internal::*;
use crate::variants::token::*;

impl<'arena> Parser<'arena> {
  pub fn macro_target_from_passthru(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Option<SourceString<'arena>> {
    if line.starts_with_seq(&[Kind(PreprocPassthru), Kind(OpenBracket)]) {
      let placeholder = line.consume_current().unwrap();
      line.discard(1); // open bracket
      let index: usize = placeholder.lexeme[1..6].parse().unwrap();
      let src_len = placeholder.loc.size();
      let mut restored = BumpString::with_capacity_in(src_len as usize, self.bump);
      let content = self.ctx.passthrus[index].take().unwrap();
      for text in content.plain_text().iter() {
        restored.push_str(text);
      }
      Some(SourceString::new(restored, placeholder.loc))
    } else {
      None
    }
  }
}

#[derive(Debug)]
pub struct Accum<'arena> {
  pub inlines: InlineNodes<'arena>,
  pub text: CollectText<'arena>,
}

impl<'arena> Accum<'arena> {
  pub fn commit(&mut self) {
    self.text.commit_inlines(&mut self.inlines);
  }

  pub fn push_node(&mut self, node: Inline<'arena>, loc: SourceLocation) {
    self.commit();
    self.inlines.push(InlineNode::new(node, loc));
    self.text.loc = loc.clamp_end();
  }

  pub fn push_emdash(&mut self, token: Token, next_token: Option<&mut Token>) {
    let last_char = self.text.str().chars().next_back();
    let next_char = next_token.as_ref().and_then(|t| t.lexeme.chars().next());
    match (last_char, next_char) {
      (Some(c), Some(n)) if is_word_char(c) && is_word_char(n) => {
        self.push_node(Symbol(SymbolKind::EmDash), token.loc);
      }
      (None, None) => self.push_text_token(&token),
      (None | Some(' '), None | Some(' ')) => {
        let mut loc = token.loc;
        if last_char.is_some() {
          loc.start -= 1;
          self.text.drop_last(1);
        }
        self.push_node(Symbol(SymbolKind::SpacedEmDash), loc.incr_end());
        if let Some(next_token) = next_token {
          next_token.drop_leading_bytes(1);
        }
      }
      _ => self.push_text_token(&token),
    }
  }

  pub fn pop_node(&mut self) {
    self.inlines.pop();
  }

  pub fn maybe_push_joining_newline(&mut self, lines: &ContiguousLines<'arena>) {
    if !lines.is_empty() {
      self.commit();
      self.text.loc.end += 1;
      self.push_node(Inline::Newline, self.text.loc);
    }
  }

  pub fn trimmed_inlines(mut self) -> InlineNodes<'arena> {
    if self.inlines.remove_trailing_line_comment() {
      self.inlines.remove_trailing_newline();
      if matches!(
        self.inlines.last().map(|n| &n.content),
        Some(Inline::Discarded)
      ) {
        self.inlines.pop();
      }
      self.trimmed_inlines()
    } else {
      self.inlines
    }
  }

  #[inline(always)]
  pub fn push_text_token(&mut self, token: &Token) {
    if self.text.loc.end == token.loc.start {
      self.text.push_token(token);
    } else {
      // happens when ifdefs cause lines to be skipped
      self.text.commit_inlines(&mut self.inlines);
      self.text.push_token(token);
      self.text.loc = token.loc;
    }
  }
}

impl Substitutions {
  /// https://docs.asciidoctor.org/asciidoc/latest/pass/pass-macro/#custom-substitutions
  pub fn from_pass_macro_target(target: BumpString) -> Self {
    if target.is_empty() {
      return Substitutions::none();
    };
    let mut subs = Self::none();
    target.split(',').for_each(|value| match value {
      "c" | "specialchars" => subs.insert(Subs::SpecialChars),
      "a" | "attributes" => subs.insert(Subs::AttrRefs),
      "r" | "replacements" => subs.insert(Subs::CharReplacement),
      "m" | "macros" => subs.insert(Subs::Macros),
      "q" | "quotes" => subs.insert(Subs::InlineFormatting),
      "v" | "verbatim" => subs.insert(Subs::SpecialChars),
      "n" | "normal" => subs = Substitutions::normal(),
      // NB: rx docs say | "post replacements", but doesn't work
      "p" => subs.insert(Subs::PostReplacement),
      _ => {}
    });
    subs
  }

  pub fn from_pass_plus_token(token: &Token) -> Self {
    if token.len() == 3 {
      Substitutions::none()
    } else {
      Substitutions::only_special_chars()
    }
  }
}

pub fn extend(loc: &mut SourceLocation, nodes: &[InlineNode<'_>], adding: usize) {
  loc.end = nodes.last().map(|node| node.loc.end).unwrap_or(loc.end) + adding as u32;
}

pub fn starts_constrained(
  stop_tokens: &[TokenSpec],
  token: &Token,
  line: &Line,
  lines: &mut ContiguousLines,
) -> bool {
  debug_assert!(!stop_tokens.is_empty());
  token.kind(stop_tokens.last().unwrap().token_kind())
    && (line.terminates_constrained(stop_tokens) || lines.terminates_constrained(stop_tokens))
}

pub fn starts_unconstrained(
  stop_tokens: &[TokenSpec],
  token: &Token,
  line: &Line,
  lines: &ContiguousLines,
) -> bool {
  debug_assert!(!stop_tokens.is_empty());
  token.kind(stop_tokens[0].token_kind())
    && (stop_tokens.len() < 2 || line.current_is(stop_tokens[1].token_kind()))
    && contains_seq(stop_tokens, line, lines)
}

pub fn contains_seq(seq: &[TokenSpec], line: &Line, lines: &ContiguousLines) -> bool {
  line.contains_seq(seq) || lines.contains_seq(seq)
}

pub fn contains_len(kind: TokenKind, len: usize, line: &Line, lines: &ContiguousLines) -> bool {
  line.contains_len(kind, len) || lines.contains_len(kind, len)
}

pub fn finish_macro<'arena>(
  line: &Line<'arena>,
  loc: &mut SourceLocation,
  line_end: SourceLocation,
  text: &mut CollectText<'arena>,
) {
  if let Some(cur_location) = line.loc() {
    loc.extend(cur_location);
    text.loc = loc.clamp_end();
    loc.end -= 1; // parsing attr list moves us one past end of macro
  } else {
    loc.extend(line_end);
    text.loc = loc.clamp_end();
  }
}

fn is_word_char(c: char) -> bool {
  c.is_alphanumeric() || c == '_'
}

lazy_static! {
  pub static ref EMAIL_RE: Regex = Regex::new(
    r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
  )
  .unwrap();
}
