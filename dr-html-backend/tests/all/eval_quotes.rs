use test_utils::*;

assert_html!(
  quote_cite,
  adoc! {r#"
    [quote,,cite]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">&#8212; cite</div>
    </div>
  "#}
);

assert_html!(
  quote_source,
  adoc! {r#"
    [quote,source]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">&#8212; source</div>
    </div>
  "#}
);

assert_html!(
  quote_source_location,
  adoc! {r#"
    [quote,source,location]
    foo bar
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>foo bar</blockquote>
      <div class="attribution">
        &#8212; source<br>
        <cite>location</cite>
      </div>
    </div>
  "#}
);

assert_html!(
  complex_quote_example,
  adoc! {r#"
    .After landing the cloaked Klingon bird of prey in Golden Gate park:
    [quote,Captain James T. Kirk,Star Trek IV: The Voyage Home]
    Everybody remember where we parked.
  "#},
  html! {r#"
    <div class="quoteblock">
      <div class="title">After landing the cloaked Klingon bird of prey in Golden Gate park:</div>
      <blockquote>
        Everybody remember where we parked.
      </blockquote>
      <div class="attribution">
        &#8212; Captain James T. Kirk<br>
        <cite>Star Trek IV: The Voyage Home</cite>
      </div>
    </div>
  "#}
);

assert_html!(
  quoted_paragraph,
  adoc! {r#"
    "I hold it that a little rebellion now and then is a good thing,
    and as necessary in the political world as storms in the physical."
    -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        I hold it that a little rebellion now and then is a good thing, and as necessary in the political world as storms in the physical.
      </blockquote>
      <div class="attribution">
        &#8212; Thomas Jefferson<br>
        <cite>Papers of Thomas Jefferson: Volume 11</cite>
      </div>
    </div>
  "#}
);

assert_html!(
  quoted_paragraph_w_attr,
  adoc! {r#"
    "I hold it blah blah..."
    -- Thomas Jefferson https://site.com[Source]
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        I hold it blah blah&#8230;&#8203;
      </blockquote>
      <div class="attribution">
        &#8212; Thomas Jefferson <a href="https://site.com">Source</a>
      </div>
    </div>
  "#}
);

assert_html!(
  cite_with_period,
  adoc! {r#"
    [quote.movie#roads,Dr. Emmett Brown]
    ____
    Roads? Where we're going, we don't need roads.
    ____
  "#},
  html! {r#"
    <div id="roads" class="quoteblock movie">
      <blockquote>
        <div class="paragraph">
          <p>Roads? Where we&#8217;re going, we don&#8217;t need roads.</p>
        </div>
      </blockquote>
      <div class="attribution">&#8212; Dr. Emmett Brown</div>
    </div>
  "#}
);
