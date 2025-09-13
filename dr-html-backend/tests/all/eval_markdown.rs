use test_utils::*;

assert_html!(
  markdown_headings,
  adoc! {r#"
    # Document Title (Level 0)

    ## Section Level 1

    ### Section Level 2

    #### Section Level 3

    ##### Section Level 4

    ###### Section Level 5
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_level_1">Section Level 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_section_level_2">Section Level 2</h3>
          <div class="sect3">
            <h4 id="_section_level_3">Section Level 3</h4>
            <div class="sect4">
              <h5 id="_section_level_4">Section Level 4</h5>
              <div class="sect5">
                <h6 id="_section_level_5">Section Level 5</h6>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  "#}
);

assert_html!(
  markdown_fenced_code_block,
  adoc! {r#"
    ```ruby
    require 'sinatra'
    ```
  "#},
  html! {r#"
    <div class="listingblock">
      <div class="content">
        <pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra'</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  markdown_simple_block_quote,
  adoc! {r#"
    > a quote
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        <div class="paragraph">
          <p>a quote</p>
        </div>
      </blockquote>
    </div>
  "#}
);

assert_html!(
  markdown_quote_w_lazy_continuation,
  adoc! {r#"
    > A famous quote.
    Some more inspiring words.
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        <div class="paragraph">
          <p>A famous quote. Some more inspiring words.</p>
        </div>
      </blockquote>
    </div>
  "#}
);

assert_html!(
  markdown_quote_implicit_attribution,
  adoc! {r#"
    > I hold it that a little rebellion now and then is a good thing,
    > and as necessary in the political world as storms in the physical.
    > -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
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
  markdown_complex_block_quote,
  adoc! {r#"
    > > What's new?
    >
    > I've got Markdown in my AsciiDoc!
    >
    > > Like what?
    >
    > * Blockquotes
    > * Headings
    > * Fenced code blocks
    >
    > > Is there more?
    >
    > Yep. AsciiDoc and Markdown share a lot of common syntax already.
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        <div class="quoteblock">
          <blockquote>
            <div class="paragraph"><p>What&#8217;s new?</p></div>
          </blockquote>
        </div>
        <div class="paragraph">
          <p>I&#8217;ve got Markdown in my AsciiDoc!</p>
        </div>
        <div class="quoteblock">
          <blockquote>
            <div class="paragraph"><p>Like what?</p></div>
          </blockquote>
        </div>
        <div class="ulist">
          <ul>
            <li><p>Blockquotes</p></li>
            <li><p>Headings</p></li>
            <li><p>Fenced code blocks</p></li>
          </ul>
        </div>
        <div class="quoteblock">
          <blockquote>
            <div class="paragraph"><p>Is there more?</p></div>
          </blockquote>
        </div>
        <div class="paragraph">
          <p>Yep. AsciiDoc and Markdown share a lot of common syntax already.</p>
        </div>
      </blockquote>
    </div>
  "#}
);

assert_html!(
  markdown_thematic_break,
  adoc! {r#"
    foo

    ---

    bar

    ***

    baz

    - - -

    jim

    * * *
    
    jam
  "#},
  html! {r#"
    <div class="paragraph"><p>foo</p></div>
    <hr>
    <div class="paragraph"><p>bar</p></div>
    <hr>
    <div class="paragraph"><p>baz</p></div>
    <hr>
    <div class="paragraph"><p>jim</p></div>
    <hr>
    <div class="paragraph"><p>jam</p></div>
  "#}
);
