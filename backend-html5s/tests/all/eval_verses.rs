use test_utils::*;

// NOTE: Jirutka backend produces different verse structure using
// div.verse-block with blockquote.verse and footer elements.

assert_html!(
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
  html_e! {r#"
    <div class="verse-block"><blockquote class="verse"><pre class="verse">The fog comes
    on little cat feet.

    It sits looking
    over harbor and city
    on silent haunches
    and then moves on.</pre><footer>&#8212; <cite>Carl Sandburg, Fog</cite></footer></blockquote></div>"#}
);

assert_html!(
  verse_paragraph,
  adoc! {r#"
    [verse,Carl Sandburg, two lines from the poem Fog]
    The fog comes
    on little cat feet.
  "#},
  html_e! {r#"<div class="verse-block"><blockquote class="verse"><pre class="verse">The fog comes
  on little cat feet.</pre><footer>&#8212; <cite>Carl Sandburg, two lines from the poem Fog</cite></footer></blockquote></div>"#}
);
