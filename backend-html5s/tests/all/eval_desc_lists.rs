// use asciidork_parser::prelude::*;
use test_utils::*;

assert_html!(
  simple_description_list_1,
  adoc! {r#"
    foo:: bar
    hash:: baz
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>foo</dt><dd>bar</dd><dt>hash</dt><dd>baz</dd></dl></div>
  "#}
);

assert_html!(
  multiple_terms,
  adoc! {r#"
    foo::
    bar:: baz
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>foo</dt><dt>bar</dt><dd>baz</dd></dl></div>
  "#}
);

assert_html!(
  glossary_dlist,
  adoc! {r#"
    [glossary]
    mud:: wet, cold dirt
  "#},
  html! {r#"
    <div class="dlist glossary"><dl class="glossary"><dt>mud</dt><dd>wet, cold dirt</dd></dl></div>
  "#}
);

assert_html!(
  simple_nested_desc_list,
  adoc! {r#"
    term1:: def1
    label1::: detail1
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>term1</dt><dd>def1<dl><dt>label1</dt><dd>detail1</dd></dl></dd></dl></div>
  "#}
);

assert_html!(
  no_term_text_but_simple_attached_block,
  adoc! {r#"
    term::
    +
    paragraph
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>term</dt><dd><p>paragraph</p></dd></dl></div>
  "#}
);

assert_html!(
  simple_description_list_2,
  adoc! {r#"
    foo:: bar
    baz:: qux
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>foo</dt><dd>bar</dd><dt>baz</dt><dd>qux</dd></dl></div>
  "#}
);

assert_html!(
  description_list_w_whitespace_para,
  adoc! {r#"
    foo::

    bar is
    so baz

    baz:: qux
  "#},
  html_e! {r#"
    <div class="dlist"><dl><dt>foo</dt><dd>bar is
    so baz</dd><dt>baz</dt><dd>qux</dd></dl></div>"#}
);

assert_html!(
  list_w_continuation,
  adoc! {r#"
    foo::
    bar so baz
    +
    and more things
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>foo</dt><dd><p>bar so baz</p><p>and more things</p></dd></dl></div>
  "#}
);

assert_html!(
  list_w_double_continuation,
  adoc! {r#"
    foo::
    bar so baz
    +
    and more things
    +
    and even more things
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>foo</dt><dd><p>bar so baz</p><p>and more things</p>
    <p>and even more things</p></dd></dl></div>
  "#}
);

assert_html!(
  mixing_lists,
  adoc! {r#"
    Dairy::
    * Milk
    * Eggs
    Bakery::
    * Bread
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>Dairy</dt><dd><ul><li>Milk</li><li>Eggs</li></ul></dd><dt>Bakery</dt><dd><ul><li>Bread</li></ul></dd></dl></div>
  "#}
);

assert_html!(
  nested_description_list,
  adoc! {r#"
    Operating Systems::
      Linux:::
        . Fedora
          * Desktop
        . Ubuntu
          * Desktop
          * Server
      BSD:::
        . FreeBSD
        . NetBSD

    Cloud Providers::
      PaaS:::
        . OpenShift
        . CloudBees
      IaaS:::
        . Amazon EC2
        . Rackspace
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>Operating Systems</dt><dd><dl><dt>Linux</dt><dd><ol class="arabic"><li>Fedora<ul><li>Desktop</li></ul></li><li>Ubuntu<ul><li>Desktop</li><li>Server</li></ul></li></ol></dd><dt>BSD</dt><dd><ol class="arabic"><li>FreeBSD</li><li>NetBSD</li></ol></dd></dl></dd><dt>Cloud Providers</dt><dd><dl><dt>PaaS</dt><dd><ol class="arabic"><li>OpenShift</li><li>CloudBees</li></ol></dd><dt>IaaS</dt><dd><ol class="arabic"><li>Amazon EC2</li><li>Rackspace</li></ol></dd></dl></dd></dl></div>
  "#}
);

assert_html!(
  literal_block_inside_desc_list,
  adoc! {r#"
    // literal block inside description list
    term::
    +
    ....
    literal, line 1

    literal, line 2
    ....
    anotherterm:: def
  "#},
  html_e! {r#"
    <div class="dlist"><dl><dt>term</dt><dd><div class="literal-block"><pre>literal, line 1

    literal, line 2</pre></div></dd><dt>anotherterm</dt><dd>def</dd></dl></div>"#}
);

assert_html!(
  trailing_continuation_desc,
  strict: false,
  adoc! {r#"
    // literal block inside description list with trailing line continuation
    term::
    +
    ....
    literal
    ....
    +
    anotherterm:: def
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>term</dt><dd><div class="literal-block"><pre>literal</pre></div></dd><dt>anotherterm</dt><dd>def</dd></dl></div>
  "#}
);

// multiple listing blocks inside description list
assert_html!(
  multiple_listing_continuations,
  adoc! {r#"
    term::
    +
    ----
    listing1
    ----
    +
    ----
    listing2
    ----
    anotherterm:: def
  "#},
  html! {r#"
    <div class="dlist"><dl><dt>term</dt><dd><div class="listing-block"><pre>listing1</pre></div>
    <div class="listing-block"><pre>listing2</pre></div></dd><dt>anotherterm</dt><dd>def</dd></dl></div>
  "#}
);

// NB: asciicoctor does not render `[grays-peak]` as the link text,
// it uses the term text `Grays Peak`, for now we're not supporting this
// see `/todo.md#differences-from-asciidoctor`
assert_html!(
  anchors_starting_desc_terms,
  adoc! {r#"
    // should discover anchor at start of description term text and register it as a reference
    Highest is <<grays-peak>>, which tops <<mount-evans>>.

    [[mount-evans,Mount Evans]]Mount Evans:: 14,271 feet
    [[grays-peak]]Grays Peak:: 14,278 feet
  "#},
  html! {r##"
    <p>Highest is <a href="#grays-peak">[grays-peak]</a>, which tops <a href="#mount-evans">Mount Evans</a>.</p>
    <div class="dlist"><dl><dt><a id="mount-evans" aria-hidden="true"></a>Mount Evans</dt><dd>14,271 feet</dd><dt><a id="grays-peak" aria-hidden="true"></a>Grays Peak</dt><dd>14,278 feet</dd></dl></div>
  "##}
);
