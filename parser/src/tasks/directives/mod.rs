mod ifdef;
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
      "ifeval::" => unimplemented!("ifeval directive"),
      _ => unreachable!("Parser::try_process_directive"),
    }
  }
}
