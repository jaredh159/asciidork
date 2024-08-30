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
  wrap_literal("<pre>foo bar\nso baz</pre>")
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
  wrap_source(
    None,
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
  source_block_from_style_no_delims,
  adoc! {r#"
    [source,ruby]
    require 'sinatra'
    get '/hi' do
      "Hello World!"
    end
  "#},
  wrap_source(
    None,
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
  wrap_source(None, "ruby", "require 'sinatra'")
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
  wrap_source(None, "java", r#"System.out.println("Hello, world!");"#)
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
  wrap_source(
    None,
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
  wrap_listing(
    None,
    raw_html! {r#"
    <pre>foo &lt;bar&gt;
    so baz</pre>
  "#}
  )
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
  wrap_listing(
    None,
    raw_html! {r#"
    <pre>foo bar
    so baz</pre>
  "#}
  )
);

assert_html!(
  source_block_id,
  adoc! {r#"
    [#hello,ruby]
    ----
    require 'sinatra'

    get '/hi' do
      "Hello World!"
    end
    ----
  "#},
  wrap_source(
    Some("hello"),
    "ruby",
    raw_html! {r#"
      require 'sinatra'

      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

// helpers

fn wrap_listing(id: Option<&str>, inner: &str) -> String {
  let div = if let Some(id) = id {
    &format!(r#"<div id="{}" class="listingblock">"#, id)
  } else {
    r#"<div class="listingblock">"#
  };
  format!(
    r#"{}<div class="content">{}</div></div>"#,
    div,
    inner.trim(),
  )
}

fn wrap_literal(inner: &str) -> String {
  format!(
    r#"<div class="literalblock"><div class="content">{}</div></div>"#,
    inner.trim(),
  )
}

fn wrap_source(id: Option<&str>, lang: &str, inner: &str) -> String {
  wrap_listing(
    id,
    &format!(
      r#"<pre class="highlight"><code class="language-{lang}" data-lang="{lang}">{}</code></pre>"#,
      inner.trim(),
    ),
  )
}
