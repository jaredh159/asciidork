mod normalize_includes;
mod process_includes;

use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    match line.current_token().unwrap().lexeme.as_str() {
      "include::" => self.try_process_include_directive(line),
      "ifdef::" => todo!(),
      "ifndef::" => todo!(),
      "ifeval::" => todo!(),
      "endif::" => todo!(),
      _ => unreachable!("Parser::try_process_directive"),
    }
  }
}
