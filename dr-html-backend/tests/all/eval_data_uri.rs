use asciidork_core::JobSettings;
use test_utils::*;

assert_html!(
  basic_embed,
  resolving: MINI_GIF,
  adoc! {r#"
    :data-uri:

    image::mini.gif[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==" alt="mini">
      </div>
    </div>
  "#}
);

assert_html!(
  no_double_encode,
  resolving: MINI_GIF,
  adoc! {r#"
    :data-uri:

    image::data:image/gif;base64,YQ==[Dot]

    :!data-uri:

    image:data:image/gif;base64,YQ==[Dot]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="data:image/gif;base64,YQ==" alt="Dot">
      </div>
    </div>
    <div class="paragraph">
      <p>
        <span class="image">
          <img src="data:image/gif;base64,YQ==" alt="Dot">
        </span>
      </p>
    </div>
  "#}
);

assert_html!(
  admonition_icons,
  resolving: b"a",
  adoc! {r#"
    :icons:
    :iconsdir: fixtures
    :icontype: gif
    :data-uri:

    [TIP]
    You can use icons for admonitions by setting the 'icons' attribute.

    :icontype: jpg

    WARNING: Never start a land war in Asia.
  "#},
  html! {r#"
    <div class="admonitionblock tip">
      <table>
        <tr>
          <td class="icon">
            <img src="data:image/gif;base64,YQ==" alt="Tip">
          </td>
          <td class="content">
            You can use icons for admonitions by setting the 'icons' attribute.
          </td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock warning">
      <table>
        <tr>
          <td class="icon">
            <img src="data:image/jpg;base64,YQ==" alt="Warning">
          </td>
          <td class="content">
            Never start a land war in Asia.
          </td>
        </tr>
      </table>
    </div>
  "#}
);

assert_html!(
  mime_types,
  resolving: b"a",
  adoc! {r#"
    :data-uri:

    image::mini.svg[]

    // inline takes precedence over data-uri
    [%inline]
    image::mini.svg[]

    image:mini.png[]

    image::noext[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="data:image/svg+xml;base64,YQ==" alt="mini">
      </div>
    </div>
    <div class="imageblock">
      <div class="content">a</div>
    </div>
    <div class="paragraph">
      <p>
        <span class="image">
          <img src="data:image/png;base64,YQ==" alt="mini">
        </span>
      </p>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="data:image/application/octet-stream;base64,YQ==" alt="noext">
      </div>
    </div>
  "#}
);

assert_html!(
  no_ext,
  resolving: b"a",
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    :data-uri:

    image::https://cats.com/cat.gif[]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="https://cats.com/cat.gif" alt="cat">
      </div>
    </div>
  "#}
);

assert_html!(
  link,
  resolving: b"a",
  adoc! {r#"
    :data-uri:

    image::mini.svg[link=http://google.com]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <a class="image" href="http://google.com">
          <img src="data:image/svg+xml;base64,YQ==" alt="mini">
        </a>
      </div>
    </div>
  "#}
);

const MINI_GIF: &[u8] = &[
  0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
  0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00,
  0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
];
