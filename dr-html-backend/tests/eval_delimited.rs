use test_utils::{adoc, html};

mod helpers;

test_eval!(
  open_block,
  adoc! {r#"
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

test_eval!(
  listing_block,
  adoc! {r#"
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

test_eval!(
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

test_eval!(
  passthrough_block_w_subs_normal,
  adoc! {r#"
    [subs=normal]
    ++++
    foo & _<bar>_
    baz
    ++++
  "#},
  html! {r#"
    foo &amp; <em>&lt;bar&gt;</em> baz
  "#}
);

test_eval!(
  example_block,
  adoc! {r#"
    ====
    foo
    ====
  "#},
  html! {r#"
    <div class="exampleblock">
      <div class="content">
        <div class="paragraph">
          <p>foo</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  delimited_quote,
  adoc! {r#"
    [quote,Monty Python and the Holy Grail]
    ____
    Dennis: Come and see the violence inherent in the system. Help! Help!

    King Arthur: Bloody peasant!
    ____
  "#},
  html! {r#"
    <div class="quoteblock">
      <blockquote>
        <div class="paragraph">
          <p>Dennis: Come and see the violence inherent in the system. Help! Help!</p>
        </div>
        <div class="paragraph">
          <p>King Arthur: Bloody peasant!</p>
        </div>
      </blockquote>
      <div class="attribution">
        &#8212; Monty Python and the Holy Grail
      </div>
    </div>
  "#}
);

test_eval!(
  nested_delimited_blocks,
  adoc! {r#"
    ****
    --
    foo
    --
    ****
  "#},
  html! {r#"
    <div class="sidebarblock">
      <div class="content">
        <div class="openblock">
          <div class="content">
            <div class="paragraph">
              <p>foo</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  basic_block_example,
  adoc! {r#"
    ****
    This is content in a sidebar block.

    image::name.png[]

    This is more content in the sidebar block.
    ****
  "#},
  html! {r#"
    <div class="sidebarblock">
      <div class="content">
        <div class="paragraph">
          <p>This is content in a sidebar block.</p>
        </div>
        <div class="imageblock">
          <div class="content">
            <img src="name.png" alt="name">
          </div>
        </div>
        <div class="paragraph">
          <p>This is more content in the sidebar block.</p>
        </div>
      </div>
    </div>
  "#}
);