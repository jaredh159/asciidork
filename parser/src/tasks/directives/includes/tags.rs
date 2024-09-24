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
  fn satisfied_by(&self, stack: &[String]) -> bool {
    match self {
      Spec::AllNonTagDirectiveLines => true,
      Spec::NoLines => false,
      Spec::AllTaggedRegions => !stack.is_empty(),
      Spec::AllUntaggedRegions => stack.is_empty(),
      Spec::Tag(tag) => stack.contains(tag),
      Spec::AllLinesExcl(tag) => !stack.contains(tag),
      Spec::TagExclAllNested(tag) => stack.last() == Some(tag),
      Spec::TagExclNested(incl, excl) => stack.contains(incl) && !stack.contains(excl),
      Spec::AllTaggedExcl(tag) => !stack.is_empty() && !stack.contains(tag),
      Spec::AllUntaggedIncl(tag) => stack.is_empty() || stack.last() == Some(tag),
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
    .split(|c| c == ',' || c == ';')
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
    src: &mut BumpVec<'arena, u8>,
  ) -> Result<()> {
    if let Some((tag, loc)) = attr_list.named_with_loc("tag") {
      let Some(selection) = parse_spec(tag) else {
        // TODO: possibly warn?
        return Ok(());
      };
      let selection = TagSpecs(vec![selection]);
      self._select_tagged_lines(selection, src, loc)
    } else if let Some((tags, loc)) = attr_list.named_with_loc("tags") {
      let selection = parse_selection(tags);
      self._select_tagged_lines(selection, src, loc)
    } else {
      Ok(())
    }
  }

  fn _select_tagged_lines(
    &mut self,
    selection: TagSpecs,
    src: &mut BumpVec<'arena, u8>,
    loc: SourceLocation,
  ) -> Result<()> {
    let mut dest = BumpVec::with_capacity_in(src.len(), self.bump);
    let mut tag_stack = Vec::with_capacity(2);
    let lines = src.split(|&c| c == b'\n');
    let strategy = selection.strategy();
    let mut expected_tags = selection.expected_tags();
    for line in lines {
      match tag_directive(line) {
        Some(TagDirective::Start(tag)) => {
          expected_tags.remove(&tag);
          tag_stack.push(tag);
        }
        Some(TagDirective::End(tag)) => {
          if tag_stack.last() == Some(&tag) {
            tag_stack.pop();
          } else {
            // TODO: emit error
          }
        }
        None => {
          if (strategy == Strategy::All
            && selection.specs().all(|spec| spec.satisfied_by(&tag_stack)))
            || (strategy == Strategy::Any
              && selection.specs().any(|spec| spec.satisfied_by(&tag_stack)))
          {
            dest.extend(line);
            dest.push(b'\n');
          }
        }
      }
    }

    std::mem::swap(src, &mut dest);

    if !expected_tags.is_empty() {
      let mut tags = expected_tags.into_iter().collect::<Vec<_>>();
      tags.sort_unstable();
      self.err_at_loc(
        format!(
          "Tag{} `{}` not found in included file",
          if tags.len() > 1 { "s" } else { "" },
          tags.join("`, `"),
        ),
        loc,
      )?;
    }

    Ok(())
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
          inside foo
          outside
        ",
      ),
      (
        simple_file(),
        "!*",
        TagSpecs(vec![AllUntaggedRegions]),
        "
          outside
          outside
        ",
      ),
      (
        simple_file(),
        "*",
        TagSpecs(vec![AllTaggedRegions]),
        "inside foo\n",
      ),
      (simple_file(), "!**", TagSpecs(vec![NoLines]), ""),
      (
        simple_file(),
        "foo",
        TagSpecs(vec![Tag("foo".to_string())]),
        "inside foo\n",
      ),
      (
        ruby_file_wrapped(),
        "init",
        TagSpecs(vec![Tag("init".to_string())]),
        "
          def initialize breed
          @breed = breed
          end
        ",
      ),
      (
        snippet_file(),
        "snippetA",
        TagSpecs(vec![Tag("snippetA".to_string())]),
        "snippetA content\n",
      ),
      (
        snippet_file(),
        "snippetA,snippetB",
        TagSpecs(vec![
          Tag("snippetA".to_string()),
          Tag("snippetB".to_string()),
        ]),
        "snippetA content\nsnippetB content\n",
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
        "  <snippet>content</snippet>\n",
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
        "let s = SS.empty;;\n",
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
        "    <p>Welcome to the club.</p>\n",
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
        "included line\r\n",
      ),
      (
        // asciidoctor no trailing newline test
        file_from("not included\ntag::include-me[]\nincluded\nend::include-me[]"),
        "include-me",
        TagSpecs(vec![Tag("include-me".to_string())]),
        "included\n",
      ),
      (
        snippet_file(),
        "snippet",
        TagSpecs(vec![Tag("snippet".to_string())]),
        indoc! {"
          snippetA content

          snippet content

          snippetB content
        "},
      ),
      (
        ruby_file_wrapped(),
        "all;!bark",
        TagSpecs(vec![TagExclNested("all".to_string(), "bark".to_string())]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          end
        "},
      ),
      (
        ruby_file_wrapped(),
        "**;!bark",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("bark".to_string()),
        ]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          end
        "},
      ),
      (
        ruby_file_wrapped(),
        "**;!init",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("init".to_string()),
        ]),
        indoc! {"
          class Dog

          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
          end
        "},
      ),
      (
        simple_file(),
        "**;!*",
        TagSpecs(vec![AllNonTagDirectiveLines, AllUntaggedRegions]),
        indoc! {"
          outside
          outside
        "},
      ),
      (
        simple_file(),
        "!*",
        TagSpecs(vec![AllUntaggedRegions]),
        indoc! {"
          outside
          outside
        "},
      ),
      (
        ruby_file(),
        "**;bark-beagle;bark-all",
        TagSpecs(vec![AllNonTagDirectiveLines]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
          end
        "},
      ),
      (
        ruby_file(),
        "!**;!init",
        TagSpecs(vec![AllTaggedExcl("init".to_string())]),
        indoc! {"
          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
        "},
      ),
      (
        ruby_file(),
        "!bark",
        TagSpecs(vec![AllLinesExcl("bark".to_string())]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          end
        "},
      ),
      (
        ruby_file(),
        "!bark;!init",
        TagSpecs(vec![
          AllLinesExcl("bark".to_string()),
          AllLinesExcl("init".to_string()),
        ]),
        indoc! {"
          class Dog
          end
        "},
      ),
      (
        ruby_file(),
        "init;**;*;!bark-other",
        TagSpecs(vec![
          AllNonTagDirectiveLines,
          AllLinesExcl("bark-other".to_string()),
        ]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          end
          end
          end
        "},
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
        indoc! {"
          a content
          b content
        "},
      ),
      (
        ruby_file_wrapped(),
        "*",
        TagSpecs(vec![AllTaggedRegions]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end

          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
          end
        "},
      ),
      (
        // include directive selects lines inside all tags except tag which is
        // negated when value of tags attr is wildcard followed by negated tag
        ruby_file_wrapped(),
        "*;!init",
        TagSpecs(vec![AllTaggedExcl("init".to_string())]),
        indoc! {"
          class Dog

          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
          end
        "},
      ),
      (
        // include directive skips all tagged regions except ones re-enabled
        // when value of tags attr is negated wildcard followed by tag name
        ruby_file(),
        "!*;init",
        TagSpecs(vec![AllUntaggedIncl("init".to_string())]),
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          end
        "},
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
        indoc! {"
          class Dog
          def initialize breed
          @breed = breed
          end
          end
        "},
      ),
      (
        // include directive includes lines inside tag except for lines
        // inside nested tags when tag is followed by negated wildcard
        ruby_file(),
        "bark;!*",
        TagSpecs(vec![TagExclAllNested("bark".to_string())]),
        "def bark\nend\n",
      ),
      (
        // include directive includes lines inside tag except for lines
        // inside nested tags when tag is followed by negated wildcard
        ruby_file(),
        "!**;bark;!*",
        TagSpecs(vec![NoLines, TagExclAllNested("bark".to_string())]),
        "def bark\nend\n",
      ),
      (
        // include directive selects lines inside tag except for lines inside
        // nested tags when tag preceded by negated double asterisk & negated wildcard
        // https://github.com/asciidoctor/asciidoctor/blob/c519d346d9b5c714b9df25e934757dad840fd997/test/reader_test.rb#L1691
        ruby_file(),
        "!**;!*;bark",
        TagSpecs(vec![NoLines, TagExclAllNested("bark".to_string())]),
        "def bark\nend\n",
      ),
      (
        // include directive does not select lines inside tag that
        // has been included then excluded
        ruby_file(),
        "!*;init;!init",
        TagSpecs(vec![AllUntaggedRegions]),
        "class Dog\nend\n",
      ),
      (
        // include directive only selects lines inside specified tag,
        // even if proceeded by negated double asterisk
        ruby_file(),
        "!**;bark",
        TagSpecs(vec![NoLines, Tag("bark".to_string())]),
        indoc! {"
          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          else
          'woof woof'
          end
          end
        "},
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
        indoc! {"
          def bark
          if @breed == 'beagle'
          'woof woof woof woof woof'
          end
          end
        "},
      ),
    ];

    for (mut file, unparsed, selection, expected) in cases.into_iter() {
      let mut parser = test_parser!("");
      assert_eq!(parse_selection(unparsed), selection);
      parser
        ._select_tagged_lines(selection, &mut file, SourceLocation::new(0, 1))
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
