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
