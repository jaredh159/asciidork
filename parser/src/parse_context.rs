use std::collections::{HashMap, HashSet};
use std::{cell::RefCell, rc::Rc};

use crate::internal::*;

#[derive(Debug)]
pub struct ParseContext<'arena> {
  pub subs: Substitutions,
  pub delimiter: Option<Delimiter>,
  pub list: ListContext,
  pub section_level: u8,
  pub leveloffset: i8,
  pub custom_line_comment: Option<SmallVec<[u8; 3]>>,
  pub anchor_ids: Rc<RefCell<HashSet<BumpString<'arena>>>>,
  /// xrefs are only used for diagnosing errors
  pub xrefs: Rc<RefCell<HashMap<BumpString<'arena>, SourceLocation>>>,
  pub can_nest_blocks: bool,
  pub saw_toc_macro: bool,
  pub bibliography_ctx: BiblioContext,
  pub table_cell_ctx: TableCellContext,
  pub inline_ctx: InlineCtx,
  pub passthrus: BumpVec<'arena, Option<InlineNodes<'arena>>>,
  pub max_include_depth: u16,
  pub ifdef_stack: BumpVec<'arena, BumpString<'arena>>,
  pub comment_delim_in_lines: bool,
  pub in_header: bool,
  pub in_markdown_blockquote: bool,
  pub attr_defs: BumpVec<'arena, AttrDef>,
  callouts: Rc<RefCell<BumpVec<'arena, Callout>>>,
}

#[derive(Debug, Clone)]
pub struct AttrDef {
  pub loc: SourceLocation,
  pub name: String,
  pub value: AttrValue,
  pub has_lbrace: bool,
  pub in_header: bool,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum InlineCtx {
  None,
  Single([TokenSpec; 1]),
  Double([TokenSpec; 2]),
}

impl InlineCtx {
  pub const fn specs(&self) -> Option<&[TokenSpec]> {
    match self {
      InlineCtx::None => None,
      InlineCtx::Single(specs) => Some(specs),
      InlineCtx::Double(specs) => Some(specs),
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TableCellContext {
  None,
  Cell,
  AsciiDocCell,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BiblioContext {
  None,
  Section,
  List,
}

impl<'arena> ParseContext<'arena> {
  pub fn new(bump: &'arena Bump) -> Self {
    ParseContext {
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      leveloffset: 0,
      can_nest_blocks: true,
      callouts: Rc::new(RefCell::new(bvec![in bump])),
      custom_line_comment: None,
      anchor_ids: Rc::new(RefCell::new(HashSet::new())),
      xrefs: Rc::new(RefCell::new(HashMap::new())),
      saw_toc_macro: false,
      bibliography_ctx: BiblioContext::None,
      table_cell_ctx: TableCellContext::None,
      passthrus: BumpVec::new_in(bump),
      inline_ctx: InlineCtx::None,
      max_include_depth: 64,
      comment_delim_in_lines: false,
      ifdef_stack: BumpVec::new_in(bump),
      attr_defs: BumpVec::new_in(bump),
      in_header: false,
      in_markdown_blockquote: false,
    }
  }

  pub fn clone_for_cell(&self, bump: &'arena Bump) -> Self {
    ParseContext {
      subs: Substitutions::default(),
      delimiter: None,
      list: ListContext::default(),
      section_level: 0,
      leveloffset: 0,
      can_nest_blocks: true,
      callouts: Rc::clone(&self.callouts),
      custom_line_comment: None,
      anchor_ids: Rc::clone(&self.anchor_ids),
      xrefs: Rc::clone(&self.xrefs),
      saw_toc_macro: false,
      bibliography_ctx: BiblioContext::None,
      table_cell_ctx: TableCellContext::AsciiDocCell,
      passthrus: BumpVec::new_in(bump),
      inline_ctx: InlineCtx::None,
      max_include_depth: 64,
      comment_delim_in_lines: false,
      ifdef_stack: BumpVec::new_in(bump),
      attr_defs: BumpVec::new_in(bump),
      in_header: false,
      in_markdown_blockquote: false,
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
      BlockContext::Paragraph if meta.attrs.has_str_positional("source") => {
        self.subs = Substitutions::verbatim();
      }
      _ => {}
    }
    // TODO: btm of https://docs.asciidoctor.org/asciidoc/latest/subs/apply-subs-to-blocks
    // says that subs element is only valid on leaf blocks
    self.subs = customize_subs::from_meta(self.subs, &meta.attrs);
    restore
  }

  pub fn parsing_adoc_cell(&self) -> bool {
    self.table_cell_ctx == TableCellContext::AsciiDocCell
  }

  pub fn parsing_simple_desc_def(&self) -> bool {
    if self.list.parsing_continuations || self.list.stack.is_empty() {
      return false;
    }
    self.parsing_description_list()
  }

  pub fn parsing_description_list_continuations(&self) -> bool {
    self.parsing_description_list() && self.list.parsing_continuations
  }

  pub fn parsing_description_list(&self) -> bool {
    self.delimiter.is_none()
      && self
        .list
        .stack
        .last()
        .is_some_and(|last| last.is_description())
  }

  pub fn within_block_comment(&self) -> bool {
    self.comment_delim_in_lines
      || self
        .delimiter
        .as_ref()
        .is_some_and(|d| d.kind == DelimiterKind::Comment)
  }
}
