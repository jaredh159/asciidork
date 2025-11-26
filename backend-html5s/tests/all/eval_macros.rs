use test_utils::*;

assert_html!(
  icon_macro_icons_disabled,
  adoc! {r#"
    :!icons:

    icon:github[]

    icon:github[alt="GitHub"]

    icon:github[link=https://github.com]

    // should not mangle icon inside link if icons are disabled
    https://github.com[icon:github[] GitHub]
  "#},
  html! {r#"
    <p><b class="icon">[github]</b></p>
    <p><b class="icon">[GitHub]</b></p>
    <p><a class="image" href="https://github.com"><b class="icon">[github]</b></a></p>
    <p><a href="https://github.com"><b class="icon">[github]</b> GitHub</a></p>
  "#}
);

assert_html!(
  icon_macro_icons_enabled,
  adoc! {r#"
    :icons:

    icon:github[]

    icon:github[alt="GitHub"]

    icon:github[width=16]

    icon:github[link=https://github.com]

    // should not mangle icon inside link if icons are disabled
    https://github.com[icon:github[] GitHub]
  "#},
  html! {r#"
    <p><img src="./images/icons/github.png" alt="github" class="icon"></p>
    <p><img src="./images/icons/github.png" alt="GitHub" class="icon"></p>
    <p><img src="./images/icons/github.png" alt="github" width="16" class="icon"></p>
    <p><a class="image" href="https://github.com"><img src="./images/icons/github.png" alt="github" class="icon"></a></p>
    <p><a href="https://github.com"><img src="./images/icons/github.png" alt="github" class="icon"> GitHub</a></p>
  "#}
);

assert_html!(
  icon_macro_font,
  adoc! {r#"
    :icons: font

    icon:github[]

    // an icon macro with a size should be interpreted as a
    // font-based icon with a size when icons=font
    icon:github[4x]

    // or named should work too
    icon:github[size=4x]

    // an icon macro with flip should be interpreted as a
    // flipped font-based icon when icons=font
    icon:shield[fw,flip=horizontal]

    // an icon macro with rotate should be interpreted as a
    // rotated font-based icon when icons=font
    icon:shield[fw,rotate=90]

    // an icon macro with a role and title should be interpreted
    // as a font-based icon with a class and title when icons=font
    icon:heart[role="red", title="Heart me"]
  "#},
  html! {r#"
    <p><i class="fa fa-github"></i></p>
    <p><i class="fa fa-github fa-4x"></i></p>
    <p><i class="fa fa-github fa-4x"></i></p>
    <p><i class="fa fa-shield fa-fw fa-flip-horizontal"></i></p>
    <p><i class="fa fa-shield fa-fw fa-rotate-90"></i></p>
    <p><i class="fa fa-heart red" title="Heart me"></i></p>
  "#}
);

assert_html!(
  icon_macro_shared_attrs,
  adoc! {r#"
    :!icons:

    icon:github[role=red, title="Hola", link="https://www.site.com", window="blank"]

    :icons: image

    icon:github[role=red, title="Hola", link="https://www.site.com", window="blank"]

    :icons: font

    icon:github[role=red, title="Hola", link="https://www.site.com", window="blank"]
  "#},
  html! {r#"
    <p><a class="image" href="https://www.site.com" target="blank"><b class="icon red" title="Hola">[github]</b></a></p>
    <p><a class="image" href="https://www.site.com" target="blank"><img src="./images/icons/github.png" alt="github" title="Hola" class="icon red"></a></p>
    <p><a class="image" href="https://www.site.com" target="blank"><i class="fa fa-github red" title="Hola"></i></a></p>
  "#}
);

assert_html!(
  keyboard_macro,
  adoc! {r#"
    Press kbd:[F11] to toggle.

    Or kbd:[Ctrl+Shift+N] for fun.
  "#},
  html! {r#"
    <p>Press <kbd class="key">F11</kbd> to toggle.</p>
    <p>Or <kbd class="keyseq"><kbd class="key">Ctrl</kbd>+<kbd class="key">Shift</kbd>+<kbd class="key">N</kbd></kbd> for fun.</p>
  "#}
);

assert_html!(
  inline_pass_macro,
  adoc! {r#"
    The text pass:[<u>underline me</u>] is underlined.

    Custom pass:q[<u>underline *me*</u>] is underlined.

    link:pass:[My Documents/report.pdf][Get Report]
  "#},
  html! {r#"
    <p>The text <u>underline me</u> is underlined.</p>
    <p>Custom <u>underline <strong>me</strong></u> is underlined.</p>
    <p><a href="My Documents/report.pdf">Get Report</a></p>
  "#}
);
