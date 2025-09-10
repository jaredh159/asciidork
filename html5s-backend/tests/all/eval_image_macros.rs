use asciidork_core::JobSettings;
use test_utils::*;

assert_html!(
  inline_image_macro,
  adoc! {r#"
    :foo: bar

    Click image:play.png[] to play the video.

    Foo image:a-b_c.png[] bar.

    image:t.svg[Custom alt]

    image:t.png[a' < b"]

    image:x.png[foo{foo}bar]
  "#},
  html! {r#"
    <p>Click <span class="image"><img src="play.png" alt="play"></span> to play the video.</p>
    <p>Foo <span class="image"><img src="a-b_c.png" alt="a b c"></span> bar.</p>
    <p><span class="image"><img src="t.svg" alt="Custom alt"></span></p>
    <p><span class="image"><img src="t.png" alt="a&#8217; &lt; b&quot;"></span></p>
    <p><span class="image"><img src="x.png" alt="foobarbar"></span></p>
  "#}
);

assert_html!(
  inline_image_macro_imagesdir,
  adoc! {r#"
    :imagesdir: path/to

    Click image:play.png[] to play the video.

    :imagesdir: https://example.com

    image:play.png[]

    :imagesdir: ./images

    image:play.png[]

    // abspath does not get imagesdir prepended
    Beware of the image:/tiger.png[tiger].

    // imagesdir not prepended to url target
    image:http://x.com/play.png[]
  "#},
  html! {r#"
    <p>Click <span class="image"><img src="path/to/play.png" alt="play"></span> to play the video.</p>
    <p><span class="image"><img src="https://example.com/play.png" alt="play"></span></p>
    <p><span class="image"><img src="./images/play.png" alt="play"></span></p>
    <p>Beware of the <span class="image"><img src="/tiger.png" alt="tiger"></span>.</p>
    <p><span class="image"><img src="http://x.com/play.png" alt="play"></span></p>
  "#}
);

// https://docs.asciidoctor.org/asciidoc/latest/macros/image-position/
assert_html!(
  image_position_frame_attrs,
  adoc! {r#"
    image::tiger.png[Tiger,200,200,float="right",align="center"]

    foo image:linux.png[Linux,150,150,float="right"] bar

    [.right.text-center]
    image::tiger.png[Tiger,200,200]

    foo image:linux.png[Linux,150,150,role=right] bar

    image:logo.png[title=Image B,role="related thumb right"]
  "#},
  html! {r#"
    <figure class="image-block right text-center">
      <img src="tiger.png" alt="Tiger" width="200" height="200">
    </figure>
    <p>foo <span class="image right"><img src="linux.png" alt="Linux" width="150" height="150"></span> bar</p>
    <figure class="image-block right text-center">
      <img src="tiger.png" alt="Tiger" width="200" height="200">
    </figure>
    <p>foo <span class="image right"><img src="linux.png" alt="Linux" width="150" height="150"></span> bar</p>
    <p><span class="image related thumb right"><img src="logo.png" alt="logo" title="Image B"></span></p>
  "#}
);

// https://docs.asciidoctor.org/asciidoc/latest/macros/image-link/
assert_html!(
  image_links,
  adoc! {r#"
    [link=https://example.org]
    image::logo.png[Logo]

    image::logo.png[Logo,link=https://example.org]

    image:apply.jpg[Apply,link=https://apply.example.org] today!

    // image::logo.png[Logo,link=https://example.org,window=_blank,opts=nofollow]
  "#},
  html! {r#"
    <figure class="image-block">
      <a class="image" href="https://example.org"><img src="logo.png" alt="Logo"></a>
    </figure>
    <figure class="image-block">
      <a class="image" href="https://example.org"><img src="logo.png" alt="Logo"></a>
    </figure>
    <p>
      <span class="image">
        <a class="image" href="https://apply.example.org">
          <img src="apply.jpg" alt="Apply">
        </a>
      </span> today!
    </p>
  "#}
);

assert_html!(
  block_image_title_rendered_below,
  adoc! {r#"
    image::flower.jpg[title="So pretty"]
  "#},
  html! {r#"
    <figure class="image-block">
      <img src="flower.jpg" alt="flower">
      <figcaption>Figure 1. So pretty</figcaption>
    </figure>
  "#}
);

// https://docs.asciidoctor.org/asciidoc/latest/macros/image-position/
assert_html!(
  image_size,
  adoc! {r#"
    image::flower.jpg[Flower,640,480]

    image::flower.jpg[alt=Flower,width=640,height=480]
  "#},
  html! {r#"
    <figure class="image-block">
      <img src="flower.jpg" alt="Flower" width="640" height="480">
    </figure>
    <figure class="image-block">
      <img src="flower.jpg" alt="Flower" width="640" height="480">
    </figure>
  "#}
);

// https://docs.asciidoctor.org/asciidoc/latest/macros/image-svg
assert_html!(
  svg_images,
  adoc! {r#"
    image::sample.svg[Static,300]

    image::sample.svg[Interactive,300,opts=interactive]

    :imagesdir: images

    image:tiger.svg[Tiger,fallback=tiger.png,opts=interactive]

    // image::sample.svg[Embedded,300,opts=inline]
  "#},
  html! {r#"
    <figure class="image-block">
      <img src="sample.svg" alt="Static" width="300">
    </figure>
    <figure class="image-block">
      <object type="image/svg+xml" data="sample.svg" width="300">
        <span class="alt">Interactive</span>
      </object>
    </figure>
    <p>
      <span class="image">
        <object type="image/svg+xml" data="images/tiger.svg">
          <img src="images/tiger.png" alt="Tiger">
        </object>
      </span>
    </p>
  "#}
);

assert_html!(
  svg_images_secure,
  |job_settings: &mut JobSettings| {
    job_settings.safe_mode = asciidork_core::SafeMode::Secure;
  },
  adoc! {r#"
    :imagesdir: images

    image:tiger.svg[Tiger,opts=interactive]
  "#},
  html! {r#"
    <p><span class="image"><img src="images/tiger.svg" alt="Tiger"></span></p>
  "#}
);

assert_html!(
  more_asciidoctor_image_tests,
  adoc! {r#"
    // escaped square bracket
    image:tiger.png[[Another\] Tiger]

    image:tiger.png[Tiger, 200, 100]

    // alt text and link
    image:tiger.png[Tiger, link="http://site.com/Tiger"]

    :imagesdir: img

    // self-referencing image with alt text
    image:tiger.png[Tiger, link=self]

    :imagesdir:

    // noopener added
    image:tiger.png[Tiger,link=http://site.com/Tiger,window=_blank]

    // named window with noopener
    image:tiger.png[Tiger,link=http://site.com/Tiger,window=name,opts=noopener]

    // nofollow
    image:tiger.png[Tiger,link=http://site.com/Tiger,opts=nofollow]

    // inline image macro w/ url target
    Beware of the image:http://example.com/images/tiger.png[tiger].

    // inline w/ float
    image:http://example.com/images/tiger.png[tiger, float="right"] Beware of the tigers!

    // target can contain space
    Beware of the image:big cats.png[] around here.

    :iconsdir: fixtures

    // image in section title, NB: our generated id differs from asciidoctor
    == image:{iconsdir}/dot.gif[dot] Title
  "#},
  html! {r#"
    <p><span class="image"><img src="tiger.png" alt="[Another] Tiger"></span></p>
    <p><span class="image"><img src="tiger.png" alt="Tiger" width="200" height="100"></span></p>
    <p>
      <span class="image">
        <a class="image" href="http://site.com/Tiger"><img src="tiger.png" alt="Tiger"></a>
      </span>
    </p>
    <p>
      <span class="image">
        <a class="image" href="img/tiger.png"><img src="img/tiger.png" alt="Tiger"></a>
      </span>
    </p>
    <p>
      <span class="image">
        <a class="image" href="http://site.com/Tiger" target="_blank" rel="noopener">
          <img src="tiger.png" alt="Tiger">
        </a>
      </span>
    </p>
    <p>
      <span class="image">
        <a class="image" href="http://site.com/Tiger" target="name" rel="noopener">
          <img src="tiger.png" alt="Tiger">
        </a>
      </span>
    </p>
    <p>
      <span class="image">
        <a class="image" href="http://site.com/Tiger" rel="nofollow">
          <img src="tiger.png" alt="Tiger">
        </a>
      </span>
    </p>
    <p>Beware of the <span class="image"><img src="http://example.com/images/tiger.png" alt="tiger"></span>.</p>
    <p>
      <span class="image right">
        <img src="http://example.com/images/tiger.png" alt="tiger">
      </span> Beware of the tigers!
    </p>
    <p>Beware of the <span class="image"><img src="big%20cats.png" alt="big cats"></span> around here.</p>
    <section class="doc-section level-1">
      <h2 id="_imagefixturesdot_gifdot_title">
        <span class="image"><img src="fixtures/dot.gif" alt="dot"></span> Title
      </h2>
    </section>
 "#}
);

assert_html!(
  asciidoctor_test_non_image_matches,
  adoc! {r#"
    // newline
    image:big
    cats.png[]

    // starts with space
    image: big cats.png[]

    // block macro found inline
    Not an inline image macro image::tiger.png[].
  "#},
  html! {r#"
    <p>image:big cats.png[]</p>
    <p>image: big cats.png[]</p>
    <p>Not an inline image macro image::tiger.png[].</p>
 "#}
);

assert_html!(
  image_macro_link_attr_ref,
  adoc! {r#"
    :foo: http://cats.com/cat.png

    image::{foo}[link={foo}]
  "#},
  html! {r#"
    <figure class="image-block">
      <a class="image" href="http://cats.com/cat.png">
        <img src="http://cats.com/cat.png" alt="cat">
      </a>
    </figure>
  "#}
);

assert_html!(
  image_macro_after_trailing_spaces,
  "<<<\n   \nimage::image_003.png[]\n",
  html! {r#"
    <div style="page-break-after: always;"></div>
    <figure class="image-block">
      <img src="image_003.png" alt="image 003">
    </figure>
  "#}
);
