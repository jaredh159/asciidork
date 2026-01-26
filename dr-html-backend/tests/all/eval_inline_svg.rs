use asciidork_core::JobSettings;
use test_utils::*;

assert_html!(
  strips_preamble,
  resolving: CIRCLE_SVG,
  adoc! {r#"
    image::sample.svg[opts=inline]
  "#},
  raw_html! {r#"
    <div class="imageblock"><div class="content"><svg
    viewBox="0 0 120 120" version="1.1"
    xmlns="http://www.w3.org/2000/svg" style="width:500px;height:500px"
    width="500px" height="500px">
      <circle cx="60" cy="60" r="50"/>
    </svg></div></div>"#}
);

assert_html!(
  w_size,
  resolving: CIRCLE_SVG,
  adoc! {r#"
    [%inline]
    image::circle.svg[Tiger,100]
  "#},
  raw_html! {r#"
    <div class="imageblock"><div class="content"><svg width="100"
    viewBox="0 0 120 120" version="1.1"
    xmlns="http://www.w3.org/2000/svg">
      <circle cx="60" cy="60" r="50"/>
    </svg></div></div>"#}
);

assert_html!(
  w_link,
  resolving: MINI_SVG,
  adoc! {r#"
    image::mini.svg[link=https://example.org,%inline]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <a class="image" href="https://example.org">
          <svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>
        </a>
      </div>
    </div>
  "#}
);

assert_html!(
  non_block,
  resolving: MINI_SVG,
  adoc! {r#"
    image:mini.svg[%inline]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>
        <span class="image">
          <svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>
        </span>
      </p>
    </div>
  "#}
);

assert_html!(
  empty,
  resolving: "",
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    image::empty.svg[nada,opts=inline]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <span class="alt">nada</span>
      </div>
    </div>
  "#}
);

assert_html!(
  missing,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    image::not-found.svg[Tiger,opts=inline]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <span class="alt">Tiger</span>
      </div>
    </div>
  "#}
);

assert_html!(
  empty_no_alt,
  resolving: "",
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    image::empty-no_alt.svg[,opts=inline]
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <span class="alt">empty no alt</span>
      </div>
    </div>
  "#}
);

assert_html!(
  incomplete,
  resolving: "<svg",
  adoc! {r#"
    image::incomplete.svg[,200,opts=inline]
  "#},
  // asciidoctor just asserts no exception, malformed html ok
  html! {r#"
    <div class="imageblock">
      <div class="content"><svg width="200"</div>
    </div>
  "#}
);

assert_html!(
  w_link_self_does_not_link,
  resolving: MINI_SVG,
  adoc! {r#"
    image::mini.svg[link=self,%inline]
  "#},
  contains:
   r#"<div class="content"><svg"#,
   r#"</svg></div"#,
);

assert_html!(
  inline_w_link_self_does_not_link,
  resolving: MINI_SVG,
  adoc! {r#"
    image:mini.svg[link=self,%inline]
  "#},
  contains:
   r#"<span class="image"><svg"#,
   r#"</svg></span"#,
);

assert_html!(
  percentage_width,
  resolving: MINI_SVG,
  adoc! {r#"
    image::mini.svg[width="50%",%inline]
  "#},
  contains: r#"<svg width="50%""#,
);

const CIRCLE_SVG: &[u8] = br#"<?xml version="1.0"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- An SVG of a black circle -->
<svg
viewBox="0 0 120 120" version="1.1"
xmlns="http://www.w3.org/2000/svg" style="width:500px;height:500px"
width="500px" height="500px">
  <circle cx="60" cy="60" r="50"/>
</svg>
"#;

const MINI_SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>"#;
