use test_utils::*;

assert_html!(
  example,
  adoc! {r#"
    // .collapsible
    .Toggle *Me*
    [%collapsible]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-open
    .Toggle Me
    [%collapsible%open]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-with-id-and-role
    .Toggle Me
    [#lorem.ipsum%collapsible]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-without-title
    [%collapsible]
    ====
    This content is revealed when the user clicks the words "Details".
    ====
  "#},
  html! {r##"
    <details><summary>Toggle <strong>Me</strong></summary><div class="content"><p>This content is revealed when the user clicks the words "Toggle Me".</p></div></details>
    <details open><summary>Toggle Me</summary><div class="content"><p>This content is revealed when the user clicks the words "Toggle Me".</p></div></details>
    <details id="lorem" class="ipsum"><summary>Toggle Me</summary><div class="content"><p>This content is revealed when the user clicks the words "Toggle Me".</p></div></details>
    <details><div class="content"><p>This content is revealed when the user clicks the words "Details".</p></div></details>
  "##}
);

assert_html!(
  image,
  adoc! {r#"
    // .with-link-and-window-blank
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", window=_blank]

    // .with-link-and-noopener
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", opts=noopener]

    // .with-link-and-nofollow
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", opts=nofollow]

    // .with-link-self
    image::sunset.jpg[link=self]

    // .with-link-none
    image::sunset.jpg[link=none]

    // .with-loading-lazy
    image::sunset.jpg[loading=lazy]

    // .html5s-image-default-link-self
    :html5s-image-default-link: self
    image::sunset.jpg[]

    // .html5s-image-default-link-self-with-link-none
    :html5s-image-default-link: self
    image::sunset.jpg[link=none]

    // .html5s-image-default-link-self-with-link-url
    :html5s-image-default-link: self
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655"]
  "#},
  html! {r##"
    <div class="image-block"><a class="image" href="http://www.flickr.com/photos/javh/5448336655" target="_blank" rel="noopener"><img src="sunset.jpg" alt="sunset"></a></div>
    <div class="image-block"><a class="image" href="http://www.flickr.com/photos/javh/5448336655" rel="noopener"><img src="sunset.jpg" alt="sunset"></a></div>
    <div class="image-block"><a class="image" href="http://www.flickr.com/photos/javh/5448336655" rel="nofollow"><img src="sunset.jpg" alt="sunset"></a></div>
    <div class="image-block"><a class="image bare" href="sunset.jpg" title="Open the image in full size" aria-label="Open the image in full size"><img src="sunset.jpg" alt="sunset"></a></div>
    <div class="image-block"><img src="sunset.jpg" alt="sunset"></div>
    <div class="image-block"><img src="sunset.jpg" alt="sunset" loading="lazy"></div>
    <div class="image-block"><a class="image bare" href="sunset.jpg" title="Open the image in full size" aria-label="Open the image in full size"><img src="sunset.jpg" alt="sunset"></a></div>
    <div class="image-block"><img src="sunset.jpg" alt="sunset"></div>
    <div class="image-block"><a class="image" href="http://www.flickr.com/photos/javh/5448336655"><img src="sunset.jpg" alt="sunset"></a></div>
  "##}
);

assert_html!(
  inline_image,
  adoc! {r#"
    // .image-with-link-and-window-blank
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", window=_blank]

    // .image-with-link-and-noopener
    // NB: jirutka adds rel="noopener" but we don't because target=_blank is not used
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", opts=noopener]

    // .with-link-and-nofollow
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", opts=nofollow]

    // .image-with-loading-lazy
    image:sunset.jpg[loading=lazy]

    // .icon-font
    :icons: font
    icon:heart[]

    // .icon-font-with-title
    :icons: font
    icon:heart[title="I <3 Asciidoctor"]

    // .icon-font-with-size
    :icons: font
    icon:shield[2x]

    // .icon-font-with-rotate
    :icons: font
    icon:shield[rotate=90]

    // .icon-font-with-flip
    :icons: font
    icon:shield[flip=vertical]
  "#},
  html! {r##"
    <p><a class="image" href="http://inkscape.org/doc/examples/tux.svg" target="_blank" rel="noopener"><img src="linux.svg" alt="linux"></a></p>
    <p><a class="image" href="http://inkscape.org/doc/examples/tux.svg"><img src="linux.svg" alt="linux"></a></p>
    <p><a class="image" href="http://inkscape.org/doc/examples/tux.svg" rel="nofollow"><img src="linux.svg" alt="linux"></a></p>
    <p><img src="sunset.jpg" alt="sunset" loading="lazy"></p>
    <p><i class="fa fa-heart"></i></p>
    <p><i class="fa fa-heart" title="I &lt;3 Asciidoctor"></i></p>
    <p><i class="fa fa-shield fa-2x"></i></p>
    <p><i class="fa fa-shield fa-rotate-90"></i></p>
    <p><i class="fa fa-shield fa-flip-vertical"></i></p>
  "##}
);

assert_html!(
  roles,
  adoc! {r#"
    // .role-line-through
    [line-through]#striked text#

    // .role-strike
    [strike]#striked text#

    // .role-del
    [del]#deleted text#

    // .role-ins
    [ins]#inserted text#
  "#},
  html! {r##"
    <p><s>striked text</s></p>
    <p><s>striked text</s></p>
    <p><del>deleted text</del></p>
    <p><ins>inserted text</ins></p>
  "##}
);

assert_html!(
  quotes_cs,
  adoc! {r#"
      :lang: cs
      "`chunky bacon`"

      '`chunky bacon`'
    "#},
  html! {r##"
      <p>&#x201e;chunky bacon&#x201c;</p>
      <p>&#x201a;chunky bacon&#x2018;</p>
    "##}
);

assert_html!(
  quotes_fi,
  adoc! {r#"
    :lang: fi
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201d;chunky bacon&#x201d;</p>
    <p>&#x2019;chunky bacon&#x2019;</p>
  "##}
);

assert_html!(
  quotes_nl,
  adoc! {r#"
    :lang: nl
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201e;chunky bacon&#x201d;</p>
    <p>&#x201a;chunky bacon&#x2019;</p>
  "##}
);

assert_html!(
  quotes_pl,
  adoc! {r#"
    :lang: pl
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201e;chunky bacon&#x201d;</p>
    <p>&#x00ab;chunky bacon&#x00bb;</p>
  "##}
);
