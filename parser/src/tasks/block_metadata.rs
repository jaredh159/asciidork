use crate::internal::*;
use crate::variants::token::*;

#[derive(Debug, Clone)]
pub struct BlockMetadata<'bmp> {
  pub title: Option<SourceString<'bmp>>,
  pub attrs: Option<AttrList<'bmp>>,
  pub start: usize,
}

impl BlockMetadata<'_> {
  pub fn block_style_or(&self, default: BlockContext) -> BlockContext {
    self
      .attrs
      .as_ref()
      .and_then(|attrs| attrs.block_style())
      .unwrap_or(default)
  }

  pub fn paragraph_context(&self, lines: &mut ContiguousLines) -> BlockContext {
    // line from block attrs takes precedence
    if let Some(block_style) = self.attrs.as_ref().and_then(|attrs| attrs.block_style()) {
      return block_style;
    }
    // handle inline admonitions, e.g. `TIP: never start a land war in asia`
    if lines
      .current()
      .map(|line| line.starts_with_seq(&[Word, Colon, Whitespace]))
      .unwrap_or(false)
    {
      let lexeme = lines.current_token().unwrap().lexeme;
      if let Some(context) = BlockContext::derive_admonition(lexeme) {
        let mut line = lines.consume_current().unwrap();
        line.discard(3); // word, colon, whitespace
        lines.restore_if_nonempty(line);
        return context;
      }
    }
    // default to pararagraph
    BlockContext::Paragraph
  }
}
