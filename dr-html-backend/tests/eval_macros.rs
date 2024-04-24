use test_utils::*;

mod helpers;

test_eval!(
  keyboard_macro,
  adoc! {r#"
    Press kbd:[F11] to toggle.

    Or kbd:[Ctrl+Shift+N] for fun.
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Press <kbd>F11</kbd> to toggle.</p>
    </div>
    <div class="paragraph">
      <p>Or <span class="keyseq"><kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>N</kbd></span> for fun.</p>
    </div>
  "#}
);

test_eval!(
  link_macros,
  adoc! {r#"
    Visit https://site.com for more.

    Or click link:report.pdf[here _son_].

    Brackets: <http://example.com> too.

    Escaped is not link: \http://nolink.com

    Email me at me@example.com as well.

    [subs=-macros]
    Not processed: https://site.com
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Visit <a href="https://site.com" class="bare">https://site.com</a> for more.</p>
    </div>
    <div class="paragraph">
      <p>Or click <a href="report.pdf">here <em>son</em></a>.</p>
    </div>
    <div class="paragraph">
      <p>Brackets: <a href="http://example.com" class="bare">http://example.com</a> too.</p>
    </div>
    <div class="paragraph">
      <p>Escaped is not link: http://nolink.com</p>
    </div>
    <div class="paragraph">
      <p>Email me at <a href="mailto:me@example.com">me@example.com</a> as well.</p>
    </div>
    <div class="paragraph">
      <p>Not processed: https://site.com</p>
    </div>
  "#}
);
