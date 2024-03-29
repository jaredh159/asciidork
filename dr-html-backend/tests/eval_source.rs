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
  src_shell(
    "ruby",
    raw_html! {r#"
      require 'sinatra'

      get '/hi' do
        "Hello World!"
      end
    "#}
  )
);

fn src_shell(lang: &str, inner: &str) -> String {
  format!(
    r#"<div class="listingblock"><div class="content"><pre class="highlight"><code class="language-{lang}" data-lang="{lang}">{}</code></pre></div></div>"#,
    inner.trim(),
  )
}
