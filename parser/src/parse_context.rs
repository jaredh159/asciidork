use crate::internal::*;

#[derive(Debug)]
pub struct ParseContext {
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
}

impl ParseContext {
  pub fn set_subs_for(&mut self, block_context: BlockContext, meta: &ChunkMeta) -> Substitutions {
    let restore = self.subs;
    match block_context {
      BlockContext::Passthrough => {
        self.subs = Substitutions::none();
      }
      BlockContext::Listing | BlockContext::Literal => {
        self.subs = Substitutions::none();
        self.subs.insert(Subs::SpecialChars);
      }
      _ => {}
    }
    self.subs = customize_subs::from_meta(self.subs, &meta.attrs);
    restore
  }
}
