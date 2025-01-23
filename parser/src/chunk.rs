use crate::internal::*;
use crate::variants::token::*;

pub trait ChunkMetaExt<'arena> {
  fn block_style_or(&self, default: BlockContext) -> BlockContext;
  fn block_paragraph_context(&self, lines: &mut ContiguousLines) -> BlockContext;
}

impl<'arena> ChunkMetaExt<'arena> for ChunkMeta<'arena> {
  fn block_style_or(&self, default: BlockContext) -> BlockContext {
    self.attrs.block_style().unwrap_or(default)
  }

  fn block_paragraph_context(&self, lines: &mut ContiguousLines) -> BlockContext {
    let uniform_indented = lines.trim_uniform_leading_whitespace();

    // line from block attrs takes precedence
    if let Some(block_style) = self.attrs.block_style() {
      return block_style;
    }

    // handle inline admonitions, e.g. `TIP: never start a land war in asia`
    if lines
      .current()
      .map(|line| line.starts_with_seq(&[Kind(Word), Kind(Colon), Kind(Whitespace)]))
      .unwrap_or(false)
    {
      let lexeme = &lines.current_token().unwrap().lexeme;
      if let Some(context) = BlockContext::derive_admonition(lexeme) {
        let mut line = lines.consume_current().unwrap();
        line.discard(3); // word, colon, whitespace
        lines.restore_if_nonempty(line);
        return context;
      }
    }

    // https://docs.asciidoctor.org/asciidoc/latest/verbatim/listing-blocks/#indent-method
    if uniform_indented || lines.current_satisfies(Line::is_indented) {
      BlockContext::Literal
    } else {
      BlockContext::Paragraph
    }
  }
}
