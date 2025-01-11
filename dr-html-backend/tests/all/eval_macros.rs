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
    <div class="paragraph">
      <p><span class="icon">[github&#93;</span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon">[GitHub&#93;</span></p>
    </div>
    <div class="paragraph">
      <p>
        <span class="icon">
          <a class="image" href="https://github.com">[github&#93;</a>
        </span>
      </p>
    </div>
    <div class="paragraph">
      <p><a href="https://github.com"><span class="icon">[github&#93;</span> GitHub</a></p>
    </div>
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
    <div class="paragraph">
      <p><span class="icon"><img src="./images/icons/github.png" alt="github"></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><img src="./images/icons/github.png" alt="GitHub"></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><img src="./images/icons/github.png" alt="github" width="16"></span></p>
    </div>
    <div class="paragraph">
      <p>
        <span class="icon">
          <a class="image" href="https://github.com">
            <img src="./images/icons/github.png" alt="github">
          </a>
        </span>
      </p>
    </div>
    <div class="paragraph">
      <p>
        <a href="https://github.com"><span class="icon">
          <img src="./images/icons/github.png" alt="github">
        </span> GitHub</a>
      </p>
    </div>
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
    <div class="paragraph">
      <p><span class="icon"><i class="fa fa-github"></i></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><i class="fa fa-github fa-4x"></i></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><i class="fa fa-github fa-4x"></i></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><i class="fa fa-shield fa-fw fa-flip-horizontal"></i></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon"><i class="fa fa-shield fa-fw fa-rotate-90"></i></span></p>
    </div>
    <div class="paragraph">
      <p><span class="icon red"><i class="fa fa-heart" title="Heart me"></i></span></p>
    </div>
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
    <div class="paragraph">
      <p>
        <span class="icon red">
          <a class="image" href="https://www.site.com" target="blank">[github&#93;</a>
        </span>
      </p>
    </div>
    <div class="paragraph">
      <p>
        <span class="icon red">
          <a class="image" href="https://www.site.com" target="blank">
            <img src="./images/icons/github.png" alt="github" title="Hola">
          </a>
        </span>
      </p>
    </div>
    <div class="paragraph">
      <p>
        <span class="icon red">
          <a class="image" href="https://www.site.com" target="blank">
            <i class="fa fa-github" title="Hola"></i>
          </a>
        </span>
      </p>
    </div>
  "#}
);

assert_html!(
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

assert_html!(
  inline_pass_macro,
  adoc! {r#"
    The text pass:[<u>underline me</u>] is underlined.

    Custom pass:q[<u>underline *me*</u>] is underlined.

    link:pass:[My Documents/report.pdf][Get Report]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>The text <u>underline me</u> is underlined.</p>
    </div>
    <div class="paragraph">
      <p>Custom <u>underline <strong>me</strong></u> is underlined.</p>
    </div>
    <div class="paragraph">
      <p><a href="My Documents/report.pdf">Get Report</a></p>
    </div>
  "#}
);
