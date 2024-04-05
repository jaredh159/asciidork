use crate::internal::*;

#[derive(Debug)]
pub struct ParseContext<'bmp> {
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
  pub custom_line_comment: Option<SmallVec<[u8; 3]>>,
  callouts: BumpVec<'bmp, Callout>,
}

impl<'bmp> ParseContext<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    ParseContext {
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      callouts: bvec![in bump],
      custom_line_comment: None,
    }
  }

  pub fn push_callout(&mut self, num: u8) -> Callout {
    if self.callouts.is_empty() {
      self
        .callouts
        .push(Callout { list_idx: 0, callout_idx: 0, num });
    } else {
      let last_callout_idx = self.callouts.len() - 1;
      let last = self.callouts[last_callout_idx];
      // sentinel for moving to next list
      if last.num == 0 {
        self.callouts[last_callout_idx].num = num;
      } else {
        self.callouts.push(Callout {
          list_idx: last.list_idx,
          callout_idx: last.callout_idx + 1,
          num,
        });
      }
    }
    *self.callouts.last().unwrap()
  }

  pub fn flush_callouts(&mut self, bump: &'bmp Bump) {
    let list_idx = if let Some(last) = self.callouts.last() {
      last.list_idx + 1
    } else {
      0
    };
    self.callouts = bvec![in bump; Callout { list_idx, callout_idx: 0, num: 0 }];
  }

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
