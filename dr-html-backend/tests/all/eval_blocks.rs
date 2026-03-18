use test_utils::{adoc, html};

assert_html!(
  sidebar_block_w_title,
  adoc! {r#"
    .Sidebar Title
    ****
    Here is the sidebar
    ****
  "#},
  html! {r#"
    <div class="sidebarblock">
      <div class="content">
        <div class="title">Sidebar Title</div>
        <div class="paragraph"><p>Here is the sidebar</p></div>
      </div>
    </div>
  "#}
);

assert_html!(
  literal_block_w_title,
  adoc! {r#"
    .Literal Title
    ....
    Here is the literal
    ....
  "#},
  html! {r#"
    <div class="literalblock">
      <div class="title">Literal Title</div>
      <div class="content"><pre>Here is the literal</pre></div>
    </div>
  "#}
);

assert_html!(
  listing_block_w_title,
  adoc! {r#"
    .Listing title
    [source,bash]
    ----
    cowsay hi
    ----
  "#},
  html! {r#"
    <div class="listingblock">
      <div class="title">Listing title</div>
      <div class="content">
        <pre class="highlight"><code class="language-bash" data-lang="bash">cowsay hi</code></pre>
      </div>
    </div>
  "#}
);

assert_html!(
  attr_w_hard_breaks,
  adoc! {r#"
    :w-breaks: foo, + \
    bar

    so {w-breaks}
  "#},
  html! {r#"
    <div class="paragraph">
      <p>so foo,<br> bar</p>
    </div>
  "#}
);

assert_html!(
  attr_w_2_hard_breaks,
  adoc! {r#"
    :w-breaks: foo, + \
    bar + \
    baz

    so {w-breaks}
  "#},
  html! {r#"
    <div class="paragraph">
      <p>so foo,<br> bar<br> baz</p>
    </div>
  "#}
);

assert_html!(
  block_macro_followed_by_comment,
  adoc! {r#"
    //
    image::b.png[B,240,180]
    //
  "#},
  html! {r#"
    <div class="imageblock">
      <div class="content">
        <img src="b.png" alt="B" width="240" height="180">
      </div>
    </div>
  "#}
);

assert_html!(
  img_macro_double_title_prefers_attr,
  adoc! {r#"
    [.left]
    .Image A
    image::a.png[A,240,180]

    // this image block macro has two titles
    [.left]
    .Image B-Title
    image::b.png[B,240,180,title=Image B-Attr]

    [.float-group]
    --
    [.left]
    .Image A
    image::a.png[A,240,180]

    [.left]
    .Image B
    image::b.png[B,240,180]
    --
  "#},
 contains:
   r#"<div class="title">Figure 2. Image B-Attr</div>"#,
   r#"<div class="title">Figure 3. Image A</div>"#,
);

assert_html!(
  title_from_attrlist_used_and_preferred,
  adoc! {r#"
    .Dot line title
    [title="From Attrlist"]
    ====
    content
    ====
  "#},
  contains: r#"<div class="title">Example 1. From Attrlist</div>"#
);

assert_html!(
  passthru_block_titles_ignored,
  adoc! {r#"
    .foo
    ++++
    bar
    ++++

    .baz
    ++++
    qux
    ++++
  "#},
  "barqux"
);

assert_html!(
  custom_subs_multi_replace,
  adoc! {r#"
    [source,java,subs="verbatim,quotes"]
    ----
    System.out.println("Hello *<name>*")
    ----
  "#},
  html! {r#"
    <div class="listingblock">
      <div class="content">
        <pre class="highlight">
          <code class="language-java" data-lang="java">System.out.println("Hello <strong>&lt;name&gt;</strong>")</code>
        </pre>
      </div>
    </div>
  "#}
);
