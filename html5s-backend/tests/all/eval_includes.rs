use asciidork_core::{JobSettings, SafeMode};
use asciidork_parser::includes::*;
use test_utils::*;

assert_html!(
  simple_include_no_newline,
  resolving: b"Line-2",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
  "#}
);

assert_html!(
  included_csv,
  resolving: b"A1,\nB1,B2",
  adoc! {r#"
    ,===
    include::some_file.adoc[]
    ,===
  "#},
  contains: r#"<p class="tableblock">A1</p>"#,
);

assert_html!(
  include_within_comment_skipped,
  resolving: b"////\nNOT-INCLUDED\n////\n",
  adoc! {r#"
    x
    ////
    include::include.adoc[]
    ////
  "#},
  r#"<div class="paragraph"><p>x</p></div>"#,
);

assert_html!(
  no_panic_include_comment,
  resolving: b"////\n////\n",
  adoc! {r#"
    x
    ////
    x

    include::include.adoc[]
    ////
  "#},
  r#"<div class="paragraph"><p>x</p></div>"#,
);

assert_html!(
  // https://github.com/jaredh159/asciidork/issues/103
  detects_delimiter_end_at_include_end,
  resolving: b"// x\n\nx\n",
  adoc! {r#"
    ----
    include::include.adoc[]
    ----

    foo bar
  "#},
  contains: "<pre>// x\n\nx</pre>",
);

assert_html!(
  // https://github.com/jaredh159/asciidork/issues/95
  handles_selection_resulting_in_only_empty_lines,
  resolving: b"// tag::x[]\n\n// end::x[]\n",
  adoc! {r#"
    == x
    include::include.adoc[tags=x]
  "#},
  contains: r#"<h2 id="_x">x</h2>"#,
);

assert_html!(
  included_csv_2,
  resolving: b"A1,\nB1,B2",
  adoc! {r#"
    [%header,format=csv]
    |===
    include::some_file.adoc[]
    |===
  "#},
  contains: r#"<th class="tableblock halign-left valign-top">A1</th>"#,
);

assert_html!(
  include_at_adoc_cell_start,
  resolving: b"included\n",
  adoc! {r#"
    |===
    a|include::some_file.adoc[]
    |===
  "#},
  contains: "included"
);

assert_html!(
  inline_include_no_newline,
  resolving: b"Line-2",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2 Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_separated_paras,
  resolving: b"included\n",
  adoc! {r#"
    para1

    include::some_file.adoc[]

    para2
  "#},
  html! {r#"
    <div class="paragraph"><p>para1</p></div>
    <div class="paragraph"><p>included</p></div>
    <div class="paragraph"><p>para2</p></div>
  "#}
);

assert_html!(
  secure_include_to_link,
  |settings: &mut JobSettings| {
    settings.safe_mode = SafeMode::Secure;
  },
  adoc! {r#"
    Line-1
    include::file.adoc[]
    Line-3

    include::with spaces.adoc[]

    include::http://a.us/b.adoc[]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 <a href="file.adoc" class="bare include">file.adoc</a> Line-3</p>
    </div>
    <div class="paragraph">
      <p><a href="with spaces.adoc" class="bare include">with spaces.adoc</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.us/b.adoc" class="bare include">http://a.us/b.adoc</a></p>
    </div>
  "#}
);

assert_html!(
  inline_include_w_newline,
  resolving: b"Line-2\n",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2 Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_strips_bom,
  resolving: [0xEF, 0xBB, 0xBF, 0xE4, 0xBA, 0xBA, 0x0A],
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 人 Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_honors_encoding,
  resolving: [0x68, 0x00, 0x69, 0x00], // <-- "hi" in UTF-16 LE
  adoc! {r#"
    Line-1
    include::some_file.adoc[encoding=utf-16]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 hi Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_err_on_missing_file,
  resolving_err: ResolveError::NotFound,
  "include::404.adoc[]",
  html! {r#"
    <div class="paragraph">
      <p>Unresolved directive in test.adoc - include::404.adoc[]</p>
    </div>
  "#}
);

assert_html!(
  include_err_on_io,
  resolving_err: ResolveError::Io("permission denied".into()),
  "include::404.adoc[]",
  html! {r#"
    <div class="paragraph">
      <p>Unresolved directive in test.adoc - include::404.adoc[]</p>
    </div>
  "#}
);

assert_html!(
  inline_include_w_2_newlines,
  resolving: b"Line-2\n\n", // <-- 2 newlines
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-3
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
    <div class="paragraph">
      <p>Line-3</p>
    </div>
  "#}
);

assert_html!(
  include_inner_para_break,
  resolving: b"Line-2\n\nLine-3",
  adoc! {r#"
    Line-1
    include::some_file.adoc[]
    Line-4
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line-1 Line-2</p>
    </div>
    <div class="paragraph">
      <p>Line-3 Line-4</p>
    </div>
  "#}
);

assert_html!(
  selecting_line_range,
  resolving: b"line1\nline2\nline3\nline4\nline5\nline6\n",
  adoc! {r#"
    include::some_file.adoc[lines=1;3;5..-1]
  "#},
  contains: "<p>line1 line3 line5 line6</p>"
);

assert_html!(
  ignores_empty_tag,
  resolving: bytes! {"
    // tag::a[]
    a
    // end::a[]
  "},
  adoc! {r#"
    ----
    include::file.rb[tag=]
    ----
  "#},
  contains: "tag::a[]"
);

assert_html!(
  ignores_empty_tags,
  resolving: bytes! {"
    // tag::a[]
    a
    // end::a[]
  "},
  adoc! {r#"
    ----
    include::file.rb[tags=]
    ----
  "#},
  contains: "tag::a[]"
);

assert_html!(
  lines_attr_overrides_tags,
  resolving: bytes! {"
    Line 1
    // tag::a[]
    Tag a
    // end::a[]
  "},
  adoc! {r#"
    include::other.adoc[lines=1,tag=a]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Line 1</p>
    </div>
  "#}
);

assert_html!(
  selecting_tags_no_error_for_missing_negated,
  resolving: TAGGED_RUBY_CLASS,
  adoc! {r#"
    ----
    include::file.rb[tags=all;!no-such-tag;!unknown-tag]
    ----
  "#},
  contains: &indoc::indoc! {r#"
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
  "#}.trim()
);

assert_html!(
  include_indentation_remove,
  resolving: TAGGED_RUBY_CLASS,
  adoc! {r#"
    ----
    include::file.rb[tags=init,indent=0]
    ----
  "#},
  contains: &indoc::indoc! {r#"
    def initialize breed
      @breed = breed
    end
  "#}.trim()
);

assert_html!(
  include_indentation_increase,
  resolving: TAGGED_RUBY_CLASS,
  adoc! {r#"
    ----
    include::file.rb[tags=init,indent=4]
    ----
  "#},
  contains: "<pre>    def initialize breed\n      @breed = breed\n    end</pre>"
);

assert_html!(
  include_leveloffset,
  resolving: bytes! {"
    = Section 2
  "},
  adoc! {r#"
    == Section 1

    include::file.adoc[leveloffset=+1]

    == Section 3
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_section_3">Section 3</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

assert_html!(
  include_leveloffset_alt,
  resolving: bytes! {"
    = Section 2
  "},
  adoc! {r#"
    == Section 1

    include::file.adoc[leveloffset=+1]
  "#},
   contains: r#"<h2 id="_section_2">Section 2</h2>"#
);

assert_html!(
  drop_line_include,
  resolving: b"not-included",
  adoc! {r#"
    :attribute-missing: drop-line

    include::{missing}.adoc[]

    spaced line

    include::{missing}.adoc[]
    next line
  "#},
  html! {r#"
    <div class="paragraph"><p>spaced line</p></div>
    <div class="paragraph"><p>next line</p></div>
  "#}
);

assert_html!(
  escaped_and_spaced_include,
  resolving: b"not-included",
  adoc! {r#"
    \include::other.adoc[]

     include::other.adoc[]
  "#},
  contains:
    "include::other.adoc[]", // <-- escaped include unprocessed
    "<pre>include::other.adoc[]</pre>" // <-- spaced include parsed as literal
);

assert_html!(
  include_within_ifdef,
  resolving: b"included\n",
  adoc! {r#"
    :defined_true:

    ifdef::defined_true[include::some_file.adoc[]]
  "#},
  contains: "included",
);

const TAGGED_RUBY_CLASS: &[u8] = b"#tag::all[]
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
";

assert_html!(
  issue_75,
  resolving: bytes! {"
    // tag::body[]
    // <-- this followed by empty line 2 below caused panic
    // tag::boilerplate[]

    // end::boilerplate[]
    // end::body[]
  "},
  adoc! {r#"
    foo

    include::some_file.adoc[tags=body]
  "#},
   r#"<div class="paragraph"><p>foo</p></div>"#
);
