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
  source_w_title,
  adoc! {r#"
    .Example
    [source,ruby]
    ----
    require 'sinatra'
    ----
  "#},
  contains:
    r#"<div class="listingblock"><div class="title">Example</div><div class="content"><pre"#
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

assert_html!(
  source_block_ruby,
  &attach_listing("[source,ruby]"),
  source::wrap("ruby", "foo")
);

assert_html!(
  source_block_not_a_lang,
  &attach_listing("[source,not-a-lang]"),
  source::wrap("not-a-lang", "foo")
);

assert_html!(
  source_block_bracket_not_a_lang_implicit,
  &attach_listing("[,not-a-lang]"),
  source::wrap("not-a-lang", "foo")
);

assert_html!(
  source_block_no_doc_lang,
  &attach_listing("[source]"),
  html! {r#"
    <div class="listingblock">
      <div class="content">
        <pre class="highlight"><code>foo</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  source_block_with_doc_attr_lang,
  adoc! {r#"
    :source-language: doc-attr-lang

    [source]
    ----
    foo
    ----
  "#},
  source::wrap("doc-attr-lang", "foo")
);

assert_html!(
  source_block_custom_id,
  &attach_listing("[#custom-id,ruby]"),
  html! {r#"
    <div id="custom-id" class="listingblock">
      <div class="content">
        <pre class="highlight"><code class="language-ruby" data-lang="ruby">foo</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  source_block_with_linenums_start,
  &attach_listing("[source%linenums=3,ruby]"),
  source::wrap("ruby", "foo")
);

assert_html!(
  source_block_with_linenums,
  &attach_listing("[source%linenums,ruby]"),
  source::wrap("ruby", "foo")
);

assert_html!(
  source_block_mixed_php,
  &attach_listing("[%mixed,php]"),
  source::wrap("php", "foo")
);

assert_html!(
  source_block_role_ruby,
  &attach_listing("[.role,ruby]"),
  html! {r#"
    <div class="listingblock role">
      <div class="content">
        <pre class="highlight"><code class="language-ruby" data-lang="ruby">foo</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  source_block_id_role_ruby,
  &attach_listing("[#id.role%opt,ruby]"),
  html! {r#"
    <div id="id" class="listingblock role">
      <div class="content">
        <pre class="highlight"><code class="language-ruby" data-lang="ruby">foo</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  source_block_source_id2_role_ruby,
  &attach_listing("[source#id2.role%opt,ruby]"),
  html! {r#"
    <div id="id2" class="listingblock role">
      <div class="content">
        <pre class="highlight"><code class="language-ruby" data-lang="ruby">foo</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  source_block_ruby_dots,
  adoc! {r#"
    [source, ruby]
    ....
    foo
    ....
  "#},
  source::wrap("ruby", "foo")
);

assert_html!(
  not_source_blocks,
  adoc! {r#"
    [,]
    ----
    foo
    ----

    []
    ----
    foo
    ----

    [listing]
    ----
    foo
    ----

    [verse]
    ----
    foo
    ----

    [example]
    ----
    foo
    ----

    [literal,ruby]
    ----
    foo
    ----

    [source]
    ====
    foo
    ====
  "#},
  html! {r#"
    <div class="listingblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="listingblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="listingblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="listingblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="listingblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="literalblock">
      <div class="content"><pre>foo</pre></div>
    </div>
    <div class="exampleblock">
      <div class="content">
        <div class="paragraph"><p>foo</p></div>
      </div>
    </div>
  "#}
);

fn attach_listing(attrs: &str) -> String {
  format!("{attrs}\n----\nfoo\n----")
}
