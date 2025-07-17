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
