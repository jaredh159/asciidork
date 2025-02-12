use crate::internal::*;

impl Parser<'_> {
  pub(crate) fn diagnose_document(&self) -> Result<()> {
    self.diagnose_invalid_xrefs()?;
    self.diagnose_toc()?;
    Ok(())
  }

  fn diagnose_invalid_xrefs(&self) -> Result<()> {
    if self.ctx.table_cell_ctx != TableCellContext::None {
      return Ok(());
    }
    for (ref_target, ref_loc) in self.ctx.xrefs.borrow().iter() {
      let Some((idx, id)) = self.target_data(ref_target) else {
        // couldn't find source idx
        self.invalid_xref(ref_target, *ref_loc)?;
        continue;
      };
      if id == "__self__" {
        continue;
      }
      let anchors = self.document.anchors.borrow();
      let Some(anchor) = anchors.get(id) else {
        self.invalid_xref(ref_target, *ref_loc)?;
        continue;
      };
      if anchor.source_idx != idx {
        // NB: here we could give a more precise error saying that
        // the anchor was found, but was in a different file
        self.invalid_xref(ref_target, *ref_loc)?;
      }
    }
    Ok(())
  }

  fn invalid_xref(&self, target: &str, loc: SourceLocation) -> Result<()> {
    self.err_at(
      format!("Invalid cross reference, no anchor found for `{target}`"),
      loc,
    )
  }

  fn target_data<'a>(&self, target: &'a str) -> Option<(u16, &'a str)> {
    if target == "#" {
      Some((0, "__self__"))
    } else if target.starts_with('#') {
      Some((0, target))
    } else if let Some(idx) = target.find('#') {
      let id = &target[idx + 1..];
      let prefix = &target[..idx];
      if file::has_adoc_ext(prefix) || file::ext(prefix).is_none() {
        self
          .lexer
          .source_idx_of_xref_target(prefix)
          .map(|src_idx| (src_idx, id))
      } else {
        Some((0, id))
      }
    } else if file::has_adoc_ext(target)
      && self.lexer.source_file_at(0).file_name().ends_with(target)
    {
      Some((0, "__self__"))
    } else {
      Some((0, target))
    }
  }

  fn diagnose_toc(&self) -> Result<()> {
    let toc_pos = self.document.toc.as_ref().map(|toc| toc.position);
    match toc_pos {
      Some(TocPosition::Macro) if !self.ctx.saw_toc_macro => {
        self.err_doc_attr(
          ":toc:",
          "Table of Contents set to `macro` but macro (`toc::[]`) not found",
        )?;
      }
      Some(TocPosition::Preamble) => match &self.document.content {
        DocContent::Blocks(_) | DocContent::Sectioned { preamble: None, .. } => {
          self.err_doc_attr(
            ":toc:",
            "Table of Contents set to `preamble` but no preamble found",
          )?;
        }
        _ => {}
      },
      _ => {}
    }
    Ok(())
  }
}
