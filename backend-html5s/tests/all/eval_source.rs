use test_utils::*;

assert_html!(
  indented_literal_block,
  " foo bar",
  html! {r#"
    <div class="literal-block"><pre>foo bar</pre></div>
  "#}
);

assert_html!(
  indented_multiline_literal_block,
  " foo bar\n so baz",
  r#"<div class="literal-block"><pre>foo bar
so baz</pre></div>"#
);

// assert_html!(
//   indented_multiline_literal_block2,
//   " a\n// b\n c",
//   r#"<div class="literal-block"><pre>a
// // b
// c</pre></div>"#
// );

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
  html_e! {r#"
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra'

    get '/hi' do
      "Hello World!"
    end</code></pre></div>"#}
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
  html_e! {
  r#"<figure class="listing-block"><figcaption>Example</figcaption><pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra'</code></pre></figure>"#}
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
  raw_html! {r#"
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">get '/hi' do
      "Hello World!"
    end</code></pre></div>"#}
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
  r#"<div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">  get '/hi' do
    "Hello World!"
  end</code></pre></div>"#
);

assert_html!(
  listing_block_newline_preservation,
  adoc! {r#"
    ----
    foo <bar>
    so baz
    ----
  "#},
  r#"<div class="listing-block"><pre>foo &lt;bar&gt;
so baz</pre></div>"#
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
  r#"<div class="listing-block"><pre>foo bar
so baz</pre></div>"#
);
