use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn insert_anchor(
    &mut self,
    id: &SourceString<'arena>,
    anchor: Anchor<'arena>,
  ) -> Result<()> {
    let mut anchors = self.document.anchors.borrow_mut();
    if anchors
      .get(&id.src)
      // NB: reparsing implicit table cell causes false dupes
      .filter(|a| a.source_loc != anchor.source_loc)
      .is_some()
    {
      self.err_at(
        if anchor.is_biblio {
          "Duplicate bibliography id"
        } else {
          "Duplicate anchor id"
        },
        id.loc.start,
        id.loc.end,
      )?;
    } else {
      anchors.insert(id.src.clone(), anchor);
    }
    Ok(())
  }

  pub(crate) fn anchor_from(
    &self,
    reftext: Option<InlineNodes<'arena>>,
    source_loc: Option<SourceLocation>,
    is_biblio: bool,
  ) -> Anchor<'arena> {
    Anchor {
      reftext,
      title: InlineNodes::new(self.bump),
      source_loc,
      source_idx: self.lexer.source_idx(),
      is_biblio,
    }
  }
}
