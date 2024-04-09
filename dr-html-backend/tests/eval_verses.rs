use test_utils::*;

mod helpers;

test_eval!(
  delimited_verse_block,
  adoc! {r#"
    [verse,Carl Sandburg,Fog]
    ____
    The fog comes
    on little cat feet.

    It sits looking
    over harbor and city
    on silent haunches
    and then moves on.
    ____
  "#},
  html! {
    r#"
      <div class="verseblock">
        <pre class="content">{}</pre>
        <div class="attribution">
          &#8212; Carl Sandburg<br>
          <cite>Fog</cite>
        </div>
      </div>
    "#,
    r#"
      The fog comes
      on little cat feet.

      It sits looking
      over harbor and city
      on silent haunches
      and then moves on.
    "#
  }
);

test_eval!(
  verse_paragraph,
  adoc! {r#"
    [verse,Carl Sandburg, two lines from the poem Fog]
    The fog comes
    on little cat feet.
  "#},
  html! {
    r#"
      <div class="verseblock">
        <pre class="content">{}</pre>
        <div class="attribution">
          &#8212; Carl Sandburg<br>
          <cite>two lines from the poem Fog</cite>
        </div>
      </div>
    "#,
    r#"
      The fog comes
      on little cat feet.
    "#
  }
);

test_eval!(
  verses_have_normal_subs_and_no_callouts,
  adoc! {r#"
    [verse]
    The fog comes <1>
  "#},
  html! {r#"
    <div class="verseblock">
      <pre class="content">The fog comes &lt;1&gt;</pre>
    </div>
  "#}
);

test_eval!(
  verse_blocks_cant_contain_blocks,
  adoc! {r#"
    [verse]
    ____
    A famous verse.

    ....
    not a literal
    ....
    ____
  "#},
  html! {
    r#"
      <div class="verseblock">
        <pre class="content">{}</pre>
      </div>
    "#,
    r#"
      A famous verse.

      ....
      not a literal
      ....
    "#
  }
);
