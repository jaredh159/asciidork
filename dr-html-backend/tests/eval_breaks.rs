use test_utils::*;

mod helpers;

test_eval!(
  basic_thematic_break,
  adoc! {r#"
    foo

    '''

    bar
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo</p>
    </div>
    <hr>
    <div class="paragraph">
      <p>bar</p>
    </div>
  "#}
);

test_eval!(
  thematic_break_w_attrs,
  adoc! {r#"
    foo

    [.fancy]
    '''
    bar
  "#},
  html! {r#"
    <div class="paragraph">
      <p>foo</p>
    </div>
    <hr class="fancy">
    <div class="paragraph">
      <p>bar</p>
    </div>
  "#}
);
