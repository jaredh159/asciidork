use crate::helpers::source;
use test_utils::{adoc_win_crlf, html, raw_html};

assert_html!(
  source_block,
  adoc_win_crlf! {r#"
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
  description_list_w_whitespace_para,
  adoc_win_crlf! {r#"
    foo::

    bar is
    so baz
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd><p>bar is so baz</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  open_block,
  adoc_win_crlf! {r#"
    --
    foo
    --
  "#},
  html! {r#"
    <div class="openblock">
      <div class="content">
        <div class="paragraph">
          <p>foo</p>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  listing_block,
  adoc_win_crlf! {r#"
    ....
    foo
    ....
  "#},
  html! {r#"
    <div class="literalblock">
      <div class="content">
        <pre>foo</pre>
      </div>
    </div>
  "#}
);

assert_html!(
  passthrough_block,
  adoc_win_crlf! {r#"
    ++++
    foo & <bar>
    ++++
  "#},
  html! {r#"
    foo & <bar>
  "#}
);

assert_html!(
  single_simple_section,
  adoc_win_crlf! {r#"
    == Section 1

    Section Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Section Content.</p>
        </div>
      </div>
    </div>
  "#}
);
