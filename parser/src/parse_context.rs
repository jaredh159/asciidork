use crate::internal::*;

#[derive(Debug, Default)]
pub struct ParseContext {
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
  pub custom_line_comment: Option<SmallVec<[u8; 3]>>,
}

impl ParseContext {
  pub fn set_subs_for(&mut self, block_context: BlockContext, meta: &ChunkMeta) -> Substitutions {
    let restore = self.subs;
    match block_context {
      BlockContext::Passthrough => {
        self.subs = Substitutions::none();
      }
      BlockContext::Listing | BlockContext::Literal => {
        self.subs = Substitutions::verbatim();
      }
      BlockContext::Paragraph if meta.attrs.as_ref().map_or(false, |a| a.is_source()) => {
        self.subs = Substitutions::verbatim();
      }
      _ => {}
    }
    // TODO: btm of https://docs.asciidoctor.org/asciidoc/latest/subs/apply-subs-to-blocks
    // says that subs element is only valid on leaf blocks
    self.subs = customize_subs::from_meta(self.subs, &meta.attrs);
    restore
  }
}
