use test_utils::{adoc, raw_html};

mod helpers;

test_eval!(
  basic_callouts,
  adoc! {r#"
    [source,ruby]
    ----
    require 'sinatra' <1>

    get '/hi' do <2> <3>
      "Hello World!"
    end
    ----
  "#},
  wrap_source(
    "ruby",
    raw_html! {r#"
      require 'sinatra' <b class="conum">(1)</b>

      get '/hi' do <b class="conum">(2)</b> <b class="conum">(3)</b>
        "Hello World!"
      end
    "#}
  )
);

test_eval!(
  xml_callouts,
  adoc! {r#"
    [source,xml]
    ----
    <section>
      <title>Section Title</title> <!--1-->
    </section>
    ----
  "#},
  wrap_source(
    "xml",
    raw_html! {r#"
      &lt;section&gt;
        &lt;title&gt;Section Title&lt;/title&gt; <b class="conum">(1)</b>
      &lt;/section&gt;
    "#}
  )
);

// behind comments: // <1> (//, #, ;;, --)
// dr renders plan <1> foo\n<2> bar as colist w/ warning
// dr warns out of order <2> foo\n<1> bar

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