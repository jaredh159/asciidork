use test_utils::{adoc, raw_html};

mod helpers;

test_eval!(
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
    "ruby",
    raw_html! {r#"
      require 'sinatra'

      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

test_eval!(
  source_block_from_style_no_delims,
  adoc! {r#"
    [source,ruby]
    require 'sinatra'
    get '/hi' do
      "Hello World!"
    end
  "#},
  wrap_source(
    "ruby",
    raw_html! {r#"
      require 'sinatra'
      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

test_eval!(
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
    "rust",
    raw_html! {r#"
      fn main() {
          println!("Hello, world!");
      }
    "#}
  )
);

test_eval!(
  listing_block_newline_preservation,
  adoc! {r#"
    ----
    foo bar
    so baz
    ----
  "#},
  wrap_listing(raw_html! {r#"
    <pre>foo bar
    so baz</pre>
  "#})
);

test_eval!(
  masquerading_listing_block_newline_preservation,
  adoc! {r#"
    [listing]
    --
    foo bar
    so baz
    --
  "#},
  wrap_listing(raw_html! {r#"
    <pre>foo bar
    so baz</pre>
  "#})
);

// helpers

fn wrap_listing(inner: &str) -> String {
  format!(
    r#"<div class="listingblock"><div class="content">{}</div></div>"#,
    inner.trim(),
  )
}

fn wrap_source(lang: &str, inner: &str) -> String {
  wrap_listing(&format!(
    r#"<pre class="highlight"><code class="language-{lang}" data-lang="{lang}">{}</code></pre>"#,
    inner.trim(),
  ))
}
