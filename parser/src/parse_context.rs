use std::collections::HashSet;

use crate::internal::*;

#[derive(Debug)]
pub struct ParseContext<'bmp> {
  pub attrs: AttrEntries,
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
  pub can_nest_blocks: bool,
  pub custom_line_comment: Option<SmallVec<[u8; 3]>>,
  pub sect_ids: HashSet<BumpString<'bmp>>,
  pub saw_toc_macro: bool,
  callouts: BumpVec<'bmp, Callout>,
}

impl<'bmp> ParseContext<'bmp> {
  pub fn new(bump: &'bmp Bump) -> Self {
    ParseContext {
      attrs: AttrEntries::new(),
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      can_nest_blocks: true,
      callouts: bvec![in bump],
      custom_line_comment: None,
      sect_ids: HashSet::new(),
      saw_toc_macro: false,
    }
  }

  pub fn push_callout(&mut self, num: Option<u8>) -> Callout {
    if self.callouts.is_empty() {
      self.callouts.push(Callout {
        list_idx: 0,
        callout_idx: 0,
        number: num.unwrap_or(1),
      });
    } else {
      let last_callout_idx = self.callouts.len() - 1;
      let last = self.callouts[last_callout_idx];
      // sentinel for moving to next list
      if last.number == 0 {
        self.callouts[last_callout_idx].number = num.unwrap_or(1);
      } else {
        self.callouts.push(Callout {
          list_idx: last.list_idx,
          callout_idx: last.callout_idx + 1,
          number: num.unwrap_or(last.number + 1),
        });
      }
    }
    *self.callouts.last().unwrap()
  }

  pub fn advance_callout_list(&mut self, bump: &'bmp Bump) {
    if let Some(last) = self.callouts.last() {
      self.callouts = bvec![in bump; Callout::new(last.list_idx + 1, 0, 0 )];
    }
  }

  pub fn get_callouts(&self, number: u8) -> SmallVec<[Callout; 4]> {
    self
      .callouts
      .iter()
      .filter(|c| c.number == number)
      .copied()
      .collect()
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
