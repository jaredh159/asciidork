use test_utils::*;

assert_html!(
  open_block,
  adoc! {r#"
    --
    foo
    --
  "#},
  html! {r#"
    <div class="open-block"><div class="content"><p>foo</p></div></div>
  "#}
);

assert_html!(
  listing_block,
  adoc! {r#"
    ....
    foo
    ....
  "#},
  html! {r#"
    <div class="literal-block"><pre>foo</pre></div>
  "#}
);

assert_html!(
  passthrough_block,
  adoc! {r#"
    ++++
    foo & <bar>
    ++++
  "#},
  html! {r#"
    foo & <bar>
  "#}
);

assert_html!(
  passthrough_block_w_subs_normal,
  adoc! {r#"
    [subs=normal]
    ++++
    foo & _<bar>_
    baz
    ++++
  "#},
  raw_html! {r#"
    foo &amp; <em>&lt;bar&gt;</em>
    baz"#}
);

assert_html!(
  example_block,
  adoc! {r#"
    .My Title
    ====
    foo
    ====
  "#},
  html! {r#"
    <figure class="example-block"><figcaption>Example 1. My Title</figcaption>
    <div class="example"><p>foo</p></div></figure>
  "#}
);

assert_html!(
  nested_example_block,
  adoc! {r#"
    ====
    ======
    foo
    ======
    ====
  "#},
  html! {r#"
    <div class="example-block"><div class="example"><div class="example-block"><div class="example"><p>foo</p></div></div></div></div>
  "#}
);

assert_html!(
  delimited_quote,
  adoc! {r#"
    [quote,Monty Python and the Holy Grail]
    ____
    Dennis: Come and see the violence inherent in the system. Help! Help!

    King Arthur: Bloody peasant!
    ____
  "#},
  html! {r#"
    <div class="quote-block"><blockquote><p>Dennis: Come and see the violence inherent in the system. Help! Help!</p>
    <p>King Arthur: Bloody peasant!</p><footer>&#8212; <cite>Monty Python and the Holy Grail</cite></footer></blockquote></div>
  "#}
);

assert_html!(
  nested_delimited_blocks,
  adoc! {r#"
    ****
    --
    foo
    --
    ****
  "#},
  html! {r#"
    <aside class="sidebar"><div class="open-block"><div class="content"><p>foo</p></div></div></aside>
  "#}
);

assert_html!(
  basic_block_example,
  adoc! {r#"
    ****
    This is content in a sidebar block.

    image::name.png[]

    This is more content in the sidebar block.
    ****
  "#},
  html! {r#"
    <aside class="sidebar"><p>This is content in a sidebar block.</p>
    <div class="image-block"><img src="name.png" alt="name"></div>
    <p>This is more content in the sidebar block.</p></aside>
  "#}
);
