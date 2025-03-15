use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub fn line_heading_level(&self, line: &Line) -> Option<u8> {
    let unadjusted = line.unadjusted_heading_level()?;
    Some(adjusted_leveloffset(
      self
        .lexer
        .leveloffset(line.first_loc().unwrap().include_depth),
      adjusted_leveloffset(self.ctx.leveloffset, unadjusted),
    ))
  }

  pub fn section_start_level(
    &self,
    lines: &ContiguousLines<'arena>,
    meta: &ChunkMeta<'arena>,
  ) -> Option<u8> {
    for line in lines.iter() {
      if line.is_block_attr_list() || line.is_chunk_title() || line.is_comment() {
        continue;
      } else if let Some(level) = self.line_heading_level(line) {
        return match meta.attrs.has_str_positional("discrete")
          || meta.attrs.has_str_positional("float")
        {
          true => None,
          false => Some(level),
        };
      } else {
        return None;
      }
    }
    None
  }

  pub fn adjust_leveloffset(leveloffset: &mut i8, value: &AttrValue) {
    match value {
      AttrValue::String(s) => {
        if let Some(add) = s.strip_prefix('+') {
          *leveloffset += add.parse::<i8>().unwrap_or(0);
        } else if let Some(sub) = s.strip_prefix('-') {
          *leveloffset -= sub.parse::<i8>().unwrap_or(0);
        } else {
          *leveloffset = s.parse::<i8>().unwrap_or(*leveloffset);
        }
      }
      AttrValue::Bool(false) => *leveloffset = 0,
      AttrValue::Bool(true) => {}
    }
  }
}

const fn adjusted_leveloffset(leveloffset: i8, heading_level: u8) -> u8 {
  if leveloffset == 0 {
    return heading_level;
  }
  let new_level = (heading_level as i8) + leveloffset;
  if new_level < 0 {
    0
  } else {
    new_level as u8
  }
}
