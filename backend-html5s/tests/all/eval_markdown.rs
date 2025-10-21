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
    <section class="doc-section level-1"><h2 id="_section_level_1">Section Level 1</h2><section class="doc-section level-2"><h3 id="_section_level_2">Section Level 2</h3><section class="doc-section level-3"><h4 id="_section_level_3">Section Level 3</h4><section class="doc-section level-4"><h5 id="_section_level_4">Section Level 4</h5><section class="doc-section level-5"><h6 id="_section_level_5">Section Level 5</h6></section></section></section></section></section>
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
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra'</code></pre></div>
  "#}
);

assert_html!(
  markdown_simple_block_quote,
  adoc! {r#"
    > a quote
  "#},
  html! {r#"
    <div class="quote-block"><blockquote><p>a quote</p></blockquote></div>
  "#}
);

assert_html!(
  markdown_quote_w_lazy_continuation,
  adoc! {r#"
    > A famous quote.
    Some more inspiring words.
  "#},
  raw_html! {r#"
    <div class="quote-block"><blockquote><p>A famous quote.
    Some more inspiring words.</p></blockquote></div>"#}
);

// assert_html!(
//   markdown_quote_implicit_attribution,
//   adoc! {r#"
//     > I hold it that a little rebellion now and then is a good thing,
//     > and as necessary in the political world as storms in the physical.
//     > -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
//   "#},
//   html! {r#"
//     <div class="quoteblock">
//       <blockquote>
//         I hold it that a little rebellion now and then is a good thing, and as necessary in the political world as storms in the physical.
//       </blockquote>
//       <div class="attribution">
//         &#8212; Thomas Jefferson<br>
//         <cite>Papers of Thomas Jefferson: Volume 11</cite>
//       </div>
//     </div>
//   "#}
// );
//
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
    <div class="quote-block"><blockquote><div class="quote-block"><blockquote><p>What&#8217;s new?</p></blockquote></div>
    <p>I&#8217;ve got Markdown in my AsciiDoc!</p>
    <div class="quote-block"><blockquote><p>Like what?</p></blockquote></div>
    <div class="ulist"><ul><li>Blockquotes</li><li>Headings</li><li>Fenced code blocks</li></ul></div>
    <div class="quote-block"><blockquote><p>Is there more?</p></blockquote></div>
    <p>Yep. AsciiDoc and Markdown share a lot of common syntax already.</p></blockquote></div>
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
    <p>foo</p>
    <hr>
    <p>bar</p>
    <hr>
    <p>baz</p>
    <hr>
    <p>jim</p>
    <hr>
    <p>jam</p>
  "#}
);
