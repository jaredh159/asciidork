use std::collections::HashSet;

use lazy_static::lazy_static;
use regex::bytes::Regex;

use crate::internal::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Spec {
  AllNonTagDirectiveLines,       // `**`
  NoLines,                       // `!**`
  AllTaggedRegions,              // `*`
  AllUntaggedRegions,            // `!*`
  Tag(String),                   // foo
  AllLinesExcl(String),          // !foo
  TagExclAllNested(String),      // foo;!*
  TagExclNested(String, String), // foo;!bar
  AllTaggedExcl(String),         // *;!foo
  AllUntaggedIncl(String),       // !*;foo
}

#[derive(Debug, PartialEq, Eq)]
enum Strategy {
  Any,
  All,
}

impl Spec {
  fn satisfied_by(&self, stack: &TagStack) -> bool {
    match self {
      Spec::AllNonTagDirectiveLines => true,
      Spec::NoLines => false,
      Spec::AllTaggedRegions => !stack.is_empty(),
      Spec::AllUntaggedRegions => stack.is_empty(),
      Spec::Tag(tag) => stack.contains(tag),
      Spec::AllLinesExcl(tag) => !stack.contains(tag),
      Spec::TagExclAllNested(tag) => stack.last_is(tag),
      Spec::TagExclNested(incl, excl) => stack.contains(incl) && !stack.contains(excl),
      Spec::AllTaggedExcl(tag) => !stack.is_empty() && !stack.contains(tag),
      Spec::AllUntaggedIncl(tag) => stack.is_empty() || stack.last_is(tag),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TagSpecs(Vec<Spec>);

impl TagSpecs {
  fn strategy(&self) -> Strategy {
    if self.0.first() == Some(&Spec::AllNonTagDirectiveLines) {
      Strategy::All
    } else if self.0.iter().all(|s| matches!(s, Spec::AllLinesExcl(_))) {
      Strategy::All
    } else {
      Strategy::Any
    }
  }

  fn expected_tags(&self) -> HashSet<String> {
    self
      .0
      .iter()
      .filter_map(|spec| match spec {
        Spec::Tag(tag) => Some(tag.clone()),
        Spec::TagExclAllNested(tag) => Some(tag.clone()),
        Spec::TagExclNested(tag, _) => Some(tag.clone()),
        Spec::AllUntaggedIncl(tag) => Some(tag.clone()),
        _ => None,
      })
      .collect()
  }

  fn push(&mut self, spec: Spec) {
    match (self.0.pop(), spec) {
      (Some(Spec::Tag(prev)), Spec::AllLinesExcl(tag)) => {
        self.0.push(Spec::TagExclNested(prev, tag)); // foo;!bar
      }
      (Some(Spec::Tag(prev)), Spec::AllUntaggedRegions) => {
        self.0.push(Spec::TagExclAllNested(prev)); // foo;!*
      }
      (Some(Spec::AllTaggedRegions), Spec::AllLinesExcl(tag)) => {
        if self.0.contains(&Spec::AllNonTagDirectiveLines) {
          self.0.push(Spec::AllLinesExcl(tag)); // *;!foo
        } else {
          self.0.push(Spec::AllTaggedExcl(tag)); // *;!foo
        }
      }
      (Some(Spec::AllUntaggedRegions), Spec::Tag(tag)) => {
        if self.0.last() == Some(&Spec::NoLines) {
          // https://github.com/asciidoctor/asciidoctor/blob/c519d346d9b5c714b9df25e934757dad840fd997/test/reader_test.rb#L1691
          self.0.push(Spec::TagExclAllNested(tag)); // !**;!*;foo
        } else {
          self.0.push(Spec::AllUntaggedIncl(tag)); // !*;foo
        }
      }
      (Some(Spec::AllNonTagDirectiveLines), Spec::Tag(_)) => {
        // asciidoctor has a FIXME by this logic but says it's always
        // been that way, so matching ¯\_(ツ)_/¯
        // https://github.com/asciidoctor/asciidoctor/blob/c519d346d9b5c714b9df25e934757dad840fd997/test/reader_test.rb#L1432
        self.0.push(Spec::AllNonTagDirectiveLines);
      }
      // `!**;!foo` means all tagged lines except those tagged `foo`
      // according to asciidoctor tests
      (Some(Spec::NoLines), Spec::AllLinesExcl(tag)) => {
        self.0.push(Spec::AllTaggedExcl(tag));
      }
      (Some(Spec::AllUntaggedIncl(prev)), Spec::AllLinesExcl(tag)) if prev == tag => {
        self.0.push(Spec::AllUntaggedRegions);
      }
      // this spec effectively nullifies all previous ones
      (_, Spec::AllNonTagDirectiveLines) => self.0 = vec![Spec::AllNonTagDirectiveLines],
      (prev, spec) => {
        prev.map(|prev| self.0.push(prev));
        self.0.push(spec);
      }
    }
  }

  fn specs(&self) -> impl Iterator<Item = &Spec> {
    self.0.iter()
  }
}

pub fn parse_spec(s: &str) -> Option<Spec> {
  match s.trim() {
    "**" => Some(Spec::AllNonTagDirectiveLines),
    "!**" => Some(Spec::NoLines),
    "*" => Some(Spec::AllTaggedRegions),
    "!*" => Some(Spec::AllUntaggedRegions),
    "" => None,
    part => {
      let (tag, negated) = if let Some(tag) = part.strip_prefix('!') {
        (tag, true)
      } else {
        (part, false)
      };
      // asciidoctor docs: "tag name must not be empty and
      // must consistexclusively of non-space chars"
      if !tag.is_empty() && !tag.contains(char::is_whitespace) {
        match negated {
          true => Some(Spec::AllLinesExcl(tag.to_string())),
          false => Some(Spec::Tag(tag.to_string())),
        }
      } else {
        None
      }
    }
  }
}

pub fn parse_selection(s: &str) -> TagSpecs {
  let mut selection = TagSpecs(Vec::new());
  s.trim()
    .split([',', ';'])
    .filter_map(parse_spec)
    .for_each(|spec| {
      selection.push(spec);
    });
  selection
}

impl<'arena> Parser<'arena> {
  pub(super) fn select_tagged_lines(
    &mut self,
    attr_list: &AttrList,
    src_path: &Path,
    src: &mut BumpVec<'arena, u8>,
  ) -> Result<()> {
    if let Some((selection, loc)) = attr_list
      .named_with_loc("tag")
      .filter(|(tag, _)| !tag.is_empty())
      .and_then(|(tag, loc)| parse_spec(tag).map(|sel| (sel, loc)))
    {
      self._select_tagged_lines(TagSpecs(vec![selection]), src, src_path, loc)
    } else if let Some((tags, loc)) = attr_list
      .named_with_loc("tags")
      .filter(|(tags, _)| !tags.is_empty())
    {
      let selection = parse_selection(tags);
      self._select_tagged_lines(selection, src, src_path, loc)
    } else {
      Ok(())
    }
  }

  fn _select_tagged_lines(
    &mut self,
    selection: TagSpecs,
    src: &mut BumpVec<'arena, u8>,
    src_path: &Path,
    attr_loc: SourceLocation,
  ) -> Result<()> {
    let mut dest = BumpVec::with_capacity_in(src.len(), self.bump);
    let mut tag_stack = TagStack::new();
    let lines = src.split(|&c| c == b'\n');
    let strategy = selection.strategy();
    let mut expected_tags = selection.expected_tags();
    let mut pushed_non_empty_line = false;
    for (idx, line) in lines.enumerate() {
      match tag_directive(line) {
        Some(TagDirective::Start(tag)) => {
          dest.extend_from_slice(b"asciidorkinclude::[false]\n");
          expected_tags.remove(&tag);
          tag_stack.push(tag, idx);
        }
        Some(TagDirective::End(tag)) => {
          dest.extend_from_slice(b"asciidorkinclude::[false]\n");
          if tag_stack.last_is(&tag) {
            tag_stack.pop();
          } else if selection.expected_tags().contains(&tag) {
            let line = nth_line(src, idx).unwrap().to_string();
            let underline_start = line.find(&tag).unwrap_or(0) as u32;
            self.err(Diagnostic {
              line_num: (idx as u32) + 1,
              line,
              message: tag_stack.last().map_or_else(
                || format!("Unexpected end tag `{tag}`"),
                |t| format!("Mismatched end tag, expected `{}` but found `{}`", t.0, tag),
              ),
              underline_start,
              underline_width: tag.len() as u32,
              source_file: SourceFile::Path(src_path.clone()),
            })?;
          }
        }
        None => {
          if (strategy == Strategy::All
            && selection.specs().all(|spec| spec.satisfied_by(&tag_stack)))
            || (strategy == Strategy::Any
              && selection.specs().any(|spec| spec.satisfied_by(&tag_stack)))
          {
            if !pushed_non_empty_line
              && (!line.is_empty() || line.iter().any(|&b| !b.is_ascii_whitespace()))
            {
              pushed_non_empty_line = true;
            }
            dest.extend(line);
            dest.push(b'\n');
          }
        }
      }
    }

    if let Some((tag, line_idx)) = tag_stack.last() {
      if selection.expected_tags().contains(tag) {
        let line = nth_line(src, *line_idx).unwrap().to_string();
        let underline_start = line.find(tag).unwrap_or(0) as u32;
        self.err(Diagnostic {
          line_num: (*line_idx as u32) + 1,
          line,
          message: format!("Tag `{tag}` was not closed"),
          underline_start,
          underline_width: tag.len() as u32,
          source_file: SourceFile::Path(src_path.clone()),
        })?;
      }
    }

    if !expected_tags.is_empty() {
      let mut tags = expected_tags.into_iter().collect::<Vec<_>>();
      tags.sort_unstable();
      self.err_at(
        format!(
          "Tag{} `{}` not found in included file",
          if tags.len() > 1 { "s" } else { "" },
          tags.join("`, `"),
        ),
        attr_loc,
      )?;
    }

    if !pushed_non_empty_line {
      dest.clear();
    }

    std::mem::swap(src, &mut dest);

    Ok(())
  }
}

fn nth_line(src: &[u8], n: usize) -> Option<&str> {
  src
    .split(|&c| c == b'\n')
    .nth(n)
    .and_then(|line| std::str::from_utf8(line).ok())
}

#[derive(Debug)]
struct TagStack(Vec<(String, usize)>);

impl TagStack {
  fn new() -> Self {
    TagStack(Vec::with_capacity(3))
  }

  fn push(&mut self, tag: String, line_idx: usize) {
    self.0.push((tag, line_idx));
  }

  fn pop(&mut self) {
    self.0.pop();
  }

  const fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  fn last_is(&self, tag: &str) -> bool {
    self.0.last().map(|(t, _)| t == tag).unwrap_or(false)
  }

  fn last(&self) -> Option<&(String, usize)> {
    self.0.last()
  }

  fn contains(&self, tag: &str) -> bool {
    self.0.iter().any(|(t, _)| t == tag)
  }
}

#[derive(Debug, PartialEq, Eq)]
enum TagDirective {
  Start(String),
  End(String),
}

lazy_static! {
  // asciidoctor: /\b(?:tag|(e)nd)::(\S+?)\[\](?=$|[ \r])/m
  static ref TAG_DIRECTIVE_RX: Regex = Regex::new(r"\b(tag|end)::(\S+)\[\](?: |$|\r$)").unwrap();
}

fn tag_directive(tag: &[u8]) -> Option<TagDirective> {
  let captures = TAG_DIRECTIVE_RX.captures(tag)?;
  let tagname = std::str::from_utf8(captures.get(2).unwrap().as_bytes()).ok()?;
  if captures.get(1).unwrap().as_bytes() == b"end" {
    Some(TagDirective::End(tagname.to_string()))
  } else {
    Some(TagDirective::Start(tagname.to_string()))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use indoc::indoc;
  use pretty_assertions::assert_eq;
  use test_utils::*;
  use Spec::*;

  #[test]
  fn test_selection() {
    let cases = vec![
      (
        simple_file(),
        "**",
        TagSpecs(vec![AllNonTagDirectiveLines]),
        "
          outside
          asciidorkinclude::[false]
          inside foo
          asciidorkinclude::[false]
          outside
        ",
      ),
      (
        simple_file(),
        "!*",
        TagSpecs(vec![AllUntaggedRegions]),
        "
          outside
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          outside
        ",
      ),
      (
        simple_file(),
        "*",
        TagSpecs(vec![AllTaggedRegions]),
        "
          asciidorkinclude::[false]
          inside foo
          asciidorkinclude::[false]
        ",
      ),
      (simple_file(), "!**", TagSpecs(vec![NoLines]), ""),
      (
        simple_file(),
        "foo",
        TagSpecs(vec![Tag("foo".to_string())]),
        "
          asciidorkinclude::[false]
          inside foo
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file_wrapped(),
        "init",
        TagSpecs(vec![Tag("init".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
        ",
      ),
      (
        snippet_file(),
        "snippetA",
        TagSpecs(vec![Tag("snippetA".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          snippetA content
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
        ",
      ),
      (
        snippet_file(),
        "snippetA,snippetB",
        TagSpecs(vec![
          Tag("snippetA".to_string()),
          Tag("snippetB".to_string()),
        ]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          snippetA content
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          snippetB content
          asciidorkinclude::[false]
          asciidorkinclude::[false]
        ",
      ),
      (
        // asciidoctor circumfix comments 1
        file_from(indoc! {"
          <root>
            <!-- tag::snippet[] -->
            <snippet>content</snippet>
            <!-- end::snippet[] -->
          </root>
        "}),
        "snippet",
        TagSpecs(vec![Tag("snippet".to_string())]),
        "
          asciidorkinclude::[false]
            <snippet>content</snippet>
          asciidorkinclude::[false]
        ",
      ),
      (
        // asciidoctor circumfix comments 2
        file_from(indoc! {"
          (* tag::snippet[] *)
          let s = SS.empty;;
          (* end::snippet[] *)
        "}),
        "snippet",
        TagSpecs(vec![Tag("snippet".to_string())]),
        "
          asciidorkinclude::[false]
          let s = SS.empty;;
          asciidorkinclude::[false]
        ",
      ),
      (
        // asciidoctor circumfix comments 3
        file_from(indoc! {"
          const element = (
            <div>
              <h1>Hello, Programmer!</h1>
              <!-- tag::snippet[] -->
              <p>Welcome to the club.</p>
              <!-- end::snippet[] -->
            </div>
          )
        "}),
        "snippet",
        TagSpecs(vec![Tag("snippet".to_string())]),
        "
          asciidorkinclude::[false]
              <p>Welcome to the club.</p>
          asciidorkinclude::[false]
        ",
      ),
      (
        // asciidoctor CLRF test
        file_from(indoc! {"
          do not include\r
          tag::include-me[]\r
          included line\r
          end::include-me[]\r
          do not include\r
        "}),
        "include-me",
        TagSpecs(vec![Tag("include-me".to_string())]),
        "
          asciidorkinclude::[false]
          included line\r
          asciidorkinclude::[false]
        ",
      ),
      (
        // asciidoctor no trailing newline test
        file_from("not included\ntag::include-me[]\nincluded\nend::include-me[]"),
        "include-me",
        TagSpecs(vec![Tag("include-me".to_string())]),
        "
          asciidorkinclude::[false]
          included
          asciidorkinclude::[false]
        ",
      ),
      (
        snippet_file(),
        "snippet",
        TagSpecs(vec![Tag("snippet".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          snippetA content
          asciidorkinclude::[false]

          snippet content

          asciidorkinclude::[false]
          snippetB content
          asciidorkinclude::[false]
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file_wrapped(),
        "all;!bark",
        TagSpecs(vec![TagExclNested("all".to_string(), "bark".to_string())]),
        "
          asciidorkinclude::[false]
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file_wrapped(),
        "**;!bark",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("bark".to_string()),
        ]),
        "
          asciidorkinclude::[false]
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file_wrapped(),
        "**;!init",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("init".to_string()),
        ]),
        "
          asciidorkinclude::[false]
          class Dog
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]

          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        simple_file(),
        "**;!*",
        TagSpecs(vec![AllNonTagDirectiveLines, AllUntaggedRegions]),
        "
          outside
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          outside
        ",
      ),
      (
        simple_file(),
        "!*",
        TagSpecs(vec![AllUntaggedRegions]),
        "
          outside
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          outside
        ",
      ),
      (
        ruby_file(),
        "**;bark-beagle;bark-all",
        TagSpecs(vec![AllNonTagDirectiveLines]),
        "
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        ruby_file(),
        "!**;!init",
        TagSpecs(vec![AllTaggedExcl("init".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file(),
        "!bark",
        TagSpecs(vec![AllLinesExcl("bark".to_string())]),
        "
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        ruby_file(),
        "!bark;!init",
        TagSpecs(vec![
          AllLinesExcl("bark".to_string()),
          AllLinesExcl("init".to_string()),
        ]),
        "
          class Dog
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        ruby_file(),
        "init;**;*;!bark-other",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("bark-other".to_string()),
        ]),
        "
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        // "selects lines between tags when tags is wildcard"
        file_from(indoc! {"
          # tag::a[]
          a content
          # end::a[]

          # tag::b[]
          b content
          # end::b[]
        "}),
        "*",
        TagSpecs(vec![AllTaggedRegions]),
        "
          asciidorkinclude::[false]
          a content
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          b content
          asciidorkinclude::[false]
        ",
      ),
      (
        ruby_file_wrapped(),
        "*",
        TagSpecs(vec![AllTaggedRegions]),
        "
          asciidorkinclude::[false]
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]

          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive selects lines inside all tags except tag which is
        // negated when value of tags attr is wildcard followed by negated tag
        ruby_file_wrapped(),
        "*;!init",
        TagSpecs(vec![AllTaggedExcl("init".to_string())]),
        "
          asciidorkinclude::[false]
          class Dog
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]

          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive skips all tagged regions except ones re-enabled
        // when value of tags attr is negated wildcard followed by tag name
        ruby_file(),
        "!*;init",
        TagSpecs(vec![AllUntaggedIncl("init".to_string())]),
        "
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        // include directive skips all tagged regions except ones re-enabled
        // when value of tags attr is negated wildcard followed by tag name
        ruby_file(),
        "**;!*;init",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllUntaggedIncl("init".to_string()),
        ]),
        "
          class Dog
          asciidorkinclude::[false]
          def initialize breed
          @breed = breed
          end
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        // include directive includes lines inside tag except for lines
        // inside nested tags when tag is followed by negated wildcard
        ruby_file(),
        "bark;!*",
        TagSpecs(vec![TagExclAllNested("bark".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive includes lines inside tag except for lines
        // inside nested tags when tag is followed by negated wildcard
        ruby_file(),
        "!**;bark;!*",
        TagSpecs(vec![NoLines, TagExclAllNested("bark".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive selects lines inside tag except for lines inside
        // nested tags when tag preceded by negated double asterisk & negated wildcard
        // https://github.com/asciidoctor/asciidoctor/blob/c519d346d9b5c714b9df25e934757dad840fd997/test/reader_test.rb#L1691
        ruby_file(),
        "!**;!*;bark",
        TagSpecs(vec![NoLines, TagExclAllNested("bark".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive does not select lines inside tag that
        // has been included then excluded
        ruby_file(),
        "!*;init;!init",
        TagSpecs(vec![AllUntaggedRegions]),
        "
          class Dog
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
        ",
      ),
      (
        // include directive only selects lines inside specified tag,
        // even if proceeded by negated double asterisk
        ruby_file(),
        "!**;bark",
        TagSpecs(vec![NoLines, Tag("bark".to_string())]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          else
          'woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
      (
        // include directive selects lines inside specified tag
        // and ignores lines inside a negated tag' do
        ruby_file(),
        "bark;!bark-other",
        TagSpecs(vec![TagExclNested(
          "bark".to_string(),
          "bark-other".to_string(),
        )]),
        "
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          def bark
          asciidorkinclude::[false]
          if @breed == 'beagle'
          'woof woof woof woof woof'
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
          end
          asciidorkinclude::[false]
        ",
      ),
    ];

    for (mut file, unparsed, selection, expected) in cases.into_iter() {
      let mut parser = test_parser!("");
      assert_eq!(parse_selection(unparsed), selection);
      parser
        ._select_tagged_lines(selection, &mut file, &Path::new(""), loc!(0..1))
        .unwrap();
      let expected = unindent::unindent(expected);
      assert_eq!(std::str::from_utf8(&file).unwrap(), expected);
    }
  }

  #[test]
  fn test_parse_tag_selection() {
    let cases = [
      ("**", vec![AllNonTagDirectiveLines]),
      ("!**", vec![NoLines]),
      ("*", vec![AllTaggedRegions]),
      ("!*", vec![AllUntaggedRegions]),
      ("foo", vec![Tag("foo".to_string())]),
      ("!foo", vec![AllLinesExcl("foo".to_string())]),
      (
        "foo,!bar",
        vec![TagExclNested("foo".to_string(), "bar".to_string())],
      ),
      (
        "    foo; !bar  ",
        vec![TagExclNested("foo".to_string(), "bar".to_string())],
      ),
      ("foo;!*", vec![TagExclAllNested("foo".to_string())]),
      ("*;!foo", vec![AllTaggedExcl("foo".to_string())]),
      ("!*;foo", vec![AllUntaggedIncl("foo".to_string())]),
      (
        "!foo;!bar",
        vec![
          AllLinesExcl("foo".to_string()),
          AllLinesExcl("bar".to_string()),
        ],
      ),
    ];

    for (input, expected) in cases.into_iter() {
      assert_eq!(parse_selection(input), TagSpecs(expected));
    }
  }

  fn simple_file() -> BumpVec<'static, u8> {
    let content = indoc! {"
      outside
      #tag::foo[]
      inside foo
      #end::foo[]
      outside
    "};
    file_from(content)
  }

  fn ruby_file() -> BumpVec<'static, u8> {
    let content = indoc! {"
      class Dog
      #tag::init[]
      def initialize breed
      @breed = breed
      end
      #end::init[]
      #tag::bark[]
      def bark
      #tag::bark-beagle[]
      if @breed == 'beagle'
      'woof woof woof woof woof'
      #end::bark-beagle[]
      #tag::bark-other[]
      else
      'woof woof'
      #end::bark-other[]
      #tag::bark-all[]
      end
      #end::bark-all[]
      end
      #end::bark[]
      end
    "};
    file_from(content)
  }

  fn ruby_file_wrapped() -> BumpVec<'static, u8> {
    let content = indoc! {"
      #tag::all[]
      class Dog
      #tag::init[]
      def initialize breed
      @breed = breed
      end
      #end::init[]
      #tag::bark[]

      def bark
      #tag::bark-beagle[]
      if @breed == 'beagle'
      'woof woof woof woof woof'
      #end::bark-beagle[]
      #tag::bark-other[]
      else
      'woof woof'
      #end::bark-other[]
      #tag::bark-all[]
      end
      #end::bark-all[]
      end
      #end::bark[]
      end
      #end::all[]
    "};
    file_from(content)
  }

  fn snippet_file() -> BumpVec<'static, u8> {
    let content = indoc! {"
      untagged first line

      // tag::snippet[]
      // tag::snippetA[]
      snippetA content
      // end::snippetA[]

      snippet content

      // tag::snippetB[]
      snippetB content
      // end::snippetB[]
      // end::snippet[]

      untagged last line
    "};
    file_from(content)
  }

  fn file_from(str: &str) -> BumpVec<'static, u8> {
    let bytes = str.trim().as_bytes().iter().copied();
    BumpVec::from_iter_in(bytes, leaked_bump())
  }
}
