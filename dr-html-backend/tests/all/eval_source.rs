use crate::helpers::source;
use test_utils::{adoc, html, raw_html};

assert_html!(
  indented_literal_block,
  " foo bar",
  html! {r#"
    <div class="literalblock">
      <div class="content">
        <pre>foo bar</pre>
      </div>
    </div>
  "#}
);

assert_html!(
  indented_multiline_literal_block,
  " foo bar\n so baz",
  source::wrap_literal("<pre>foo bar\nso baz</pre>")
);

assert_html!(
  indented_multiline_literal_block2,
  " a\n// b\n c",
  source::wrap_literal("<pre> a\n// b\n c</pre>")
);

assert_html!(
  source_block_explicit,
  adoc! {r#"
    [source,ruby]
    ----
    require 'sinatra'

    get '/hi' do
      "Hello World!"
    end
    ----
  "#},
  source::wrap(
    "ruby",
    raw_html! {r#"
      require 'sinatra'

      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

assert_html!(
  source_block_indent_0,
  adoc! {r#"
    [source,ruby,indent=0]
    ----
      get '/hi' do
        "Hello World!"
      end
    ----
  "#},
  source::wrap(
    "ruby",
    raw_html! {r#"
      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

assert_html!(
  source_block_indent_2,
  adoc! {r#"
    [source,ruby,indent=2]
    ----
    get '/hi' do
      "Hello World!"
    end
    ----
  "#},
  contains: ">  get '/hi' do\n    \"Hello World!\"\n  end<"
);

assert_html!(
  source_block_from_style_no_delims,
  adoc! {r#"
    [source,ruby]
    require 'sinatra'
    get '/hi' do
      "Hello World!"
    end
  "#},
  source::wrap(
    "ruby",
    raw_html! {r#"
      require 'sinatra'
      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

assert_html!(
  source_block_lang_from_attr,
  adoc! {r#"
    :source-language: ruby

    ----
    require 'sinatra'
    ----
  "#},
  source::wrap("ruby", "require 'sinatra'")
);

assert_html!(
  source_block_lang_from_attr_override,
  adoc! {r#"
    :source-language: ruby

    [source,java]
    ----
    System.out.println("Hello, world!");
    ----
  "#},
  source::wrap("java", r#"System.out.println("Hello, world!");"#)
);

assert_html!(
  source_block_implicit,
  adoc! {r#"
    [,rust]
    ----
    fn main() {
        println!("Hello, world!");
    }
    ----
  "#},
  source::wrap(
    "rust",
    raw_html! {r#"
      fn main() {
          println!("Hello, world!");
      }
    "#}
  )
);

assert_html!(
  listing_block_newline_preservation,
  adoc! {r#"
    ----
    foo <bar>
    so baz
    ----
  "#},
  source::wrap_listing(raw_html! {r#"
    <pre>foo &lt;bar&gt;
    so baz</pre>
  "#})
);

assert_html!(
  masquerading_listing_block_newline_preservation,
  adoc! {r#"
    [listing]
    --
    foo bar
    so baz
    --
  "#},
  source::wrap_listing(raw_html! {r#"
    <pre>foo bar
    so baz</pre>
  "#})
);
