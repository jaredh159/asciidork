mod ifdef;
mod ifeval;
pub mod includes;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    match line.current_token().unwrap().lexeme.as_str() {
      "include::" => self.try_process_include_directive(line),
      "ifdef::" => self.try_process_ifdef_directive(true, line),
      "ifndef::" => self.try_process_ifdef_directive(false, line),
      "endif::" => self.try_process_endif_directive(line),
      "ifeval::" => self.try_process_ifeval_directive(line),
      _ => unreachable!("Parser::try_process_directive"),
    }
  }

  fn try_process_endif_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    let Some(endif_attrs) = line.directive_endif_target() else {
      return Ok(DirectiveAction::Passthrough);
    };
    let Some(expected) = self.ctx.ifdef_stack.last() else {
      self.err_line("This endif directive has no previous ifdef/ifndef", line)?;
      return Ok(DirectiveAction::Passthrough);
    };
    if !endif_attrs.is_empty() && expected != &endif_attrs {
      self.err_at_pattern(
        format!("Mismatched endif directive, expected `{}`", &expected),
        line.first_loc().unwrap(),
        &endif_attrs,
      )?;
      return Ok(DirectiveAction::Passthrough);
    }
    Ok(DirectiveAction::ReadNextLine)
  }
}
