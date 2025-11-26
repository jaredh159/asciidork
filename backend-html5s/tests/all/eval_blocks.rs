use test_utils::*;

assert_html!(
  sidebar_block_w_title,
  adoc! {r#"
    .Sidebar Title
    ****
    Here is the sidebar
    ****
  "#},
  html! {r#"
    <aside class="sidebar">
      <h6 class="block-title">Sidebar Title</h6>
      <p>Here is the sidebar</p>
    </aside>
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
   <section class="literal-block">
     <h6 class="block-title">Literal Title</h6>
     <pre>Here is the literal</pre>
   </section>
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
    <figure class="listing-block">
      <figcaption>Listing title</figcaption>
      <pre class="highlight"><code class="language-bash" data-lang="bash">cowsay hi</code></pre>
    </figure>
  "#}
);
