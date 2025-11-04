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

// assert_html!(
//   image2,
//   adoc! {r#"
//     // .html5s-image-default-link-self
//     :html5s-image-default-link: self
//     image::sunset.jpg[]
//   "#},
//   html! {r##"
//     <div class="image-block"><a class="image bare" href="sunset.jpg" title="Open the image in full size" aria-label="Open the image in full size"><img src="sunset.jpg" alt="sunset"></a></div>
//   "##} // <div class="image-block"><img src="sunset.jpg" alt="sunset"></div>
//        // <div class="image-block"><a class="image" href="http://www.flickr.com/photos/javh/5448336655"><img src="sunset.jpg" alt="sunset"></a></div>
// );
