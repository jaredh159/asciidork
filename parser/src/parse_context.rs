use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

#[derive(Debug)]
pub struct ParseContext<'arena> {
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
  pub can_nest_blocks: bool,
  pub custom_line_comment: Option<SmallVec<[u8; 3]>>,
  pub anchor_ids: Rc<RefCell<HashSet<BumpString<'arena>>>>,
  pub xrefs: Rc<RefCell<HashMap<BumpString<'arena>, SourceLocation>>>,
  pub num_footnotes: Rc<RefCell<u16>>,
  pub saw_toc_macro: bool,
  pub in_asciidoc_table_cell: bool,
  callouts: Rc<RefCell<BumpVec<'arena, Callout>>>,
}

impl<'arena> ParseContext<'arena> {
  pub fn new(bump: &'arena Bump) -> Self {
    ParseContext {
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      can_nest_blocks: true,
      callouts: Rc::new(RefCell::new(bvec![in bump])),
      custom_line_comment: None,
      anchor_ids: Rc::new(RefCell::new(HashSet::new())),
      xrefs: Rc::new(RefCell::new(HashMap::new())),
      num_footnotes: Rc::new(RefCell::new(0)),
      saw_toc_macro: false,
      in_asciidoc_table_cell: false,
    }
  }

  pub fn clone_for_cell(&self) -> Self {
    ParseContext {
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      can_nest_blocks: true,
      callouts: Rc::clone(&self.callouts),
      custom_line_comment: None,
      anchor_ids: Rc::clone(&self.anchor_ids),
      xrefs: Rc::clone(&self.xrefs),
      num_footnotes: Rc::clone(&self.num_footnotes),
      saw_toc_macro: false,
      in_asciidoc_table_cell: true,
    }
  }

  pub fn push_callout(&mut self, num: Option<u8>) -> Callout {
    let mut callouts = self.callouts.borrow_mut();
    if callouts.is_empty() {
      callouts.push(Callout {
        list_idx: 0,
        callout_idx: 0,
        number: num.unwrap_or(1),
      });
    } else {
      let last_callout_idx = callouts.len() - 1;
      let last = callouts[last_callout_idx];
      // sentinel for moving to next list
      if last.number == 0 {
        callouts[last_callout_idx].number = num.unwrap_or(1);
      } else {
        callouts.push(Callout {
          list_idx: last.list_idx,
          callout_idx: last.callout_idx + 1,
          number: num.unwrap_or(last.number + 1),
        });
      }
    }
    *callouts.last().unwrap()
  }

  pub fn advance_callout_list(&mut self, bump: &'arena Bump) {
    let last = { self.callouts.borrow().last().copied() };
    if let Some(last) = last {
      self.callouts = Rc::new(RefCell::new(
        bvec![in bump; Callout::new(last.list_idx + 1, 0, 0 )],
      ));
    }
  }

  pub fn get_callouts(&self, number: u8) -> SmallVec<[Callout; 4]> {
    self
      .callouts
      .borrow()
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
