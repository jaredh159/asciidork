use test_utils::*;

// NOTE: Jirutka backend produces cleaner list HTML:
// - No <p> wrappers around simple list items
// - Maintains <div class="ulist"> wrapper structure
// - Complex list items with multiple paragraphs may still use <p> tags

assert_html!(
  most_basic_unordered_list,
  adoc! {r#"
    * foo
    * bar
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>foo</li>
        <li>bar</li>
      </ul>
    </div>
  "#}
);

assert_html!(
  multiline_list_principle_w_indent,
  adoc! {r#"
    * foo _bar_
      so *baz*
    * two
  "#},
  html_e! {r#"
    <div class="ulist"><ul><li>foo <em>bar</em>
    so <strong>baz</strong></li><li>two</li></ul></div>"#}
);

assert_html!(
  simple_nested_list,
  adoc! {r#"
    * foo
    ** bar
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>foo
          <ul>
            <li>bar</li>
          </ul>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_w_title,
  adoc! {r#"
    .Kizmets Favorite Authors
    * Edgar Allan Poe
    * Sheri S. Tepper
    * Bill Bryson
  "#},
  html! {r#"
    <section class="ulist">
      <h6 class="block-title">Kizmets Favorite Authors</h6>
      <ul>
        <li>Edgar Allan Poe</li>
        <li>Sheri S. Tepper</li>
        <li>Bill Bryson</li>
      </ul>
    </section>
  "#}
);

assert_html!(
  list_custom_marker,
  adoc! {r#"
    [square]
    * Level 1 list item
    - Level 2 list item
    * Level 1 list item
  "#},
  html! {r#"
    <div class="ulist square"><ul class="square"><li>Level 1 list item<ul><li>Level 2 list item</li></ul></li><li>Level 1 list item</li></ul></div>
  "#}
);

assert_html!(
  list_marker_mid_override,
  adoc! {r#"
    [square]
    * squares
    ** up top
    [circle]
    *** circles
    **** down below
  "#},
  html! {r#"
    <div class="ulist square"><ul class="square"><li>squares<ul><li>up top<ul class="circle"><li>circles<ul><li>down below</li></ul></li></ul></li></ul></li></ul></div>
  "#}
);

assert_html!(
  list_numbered,
  adoc! {r#"
    1. Protons
    2. Electrons
    3. Neutrons
  "#},
  html! {r#"
    <div class="olist arabic"><ol class="arabic"><li>Protons</li><li>Electrons</li><li>Neutrons</li></ol></div>
  "#}
);

assert_html!(
  list_numbered_manual_start,
  adoc! {r#"
    4. Protons
    5. Electrons
    6. Neutrons
  "#},
  html! {r#"
    <div class="olist arabic"><ol class="arabic" start="4"><li>Protons</li><li>Electrons</li><li>Neutrons</li></ol></div>
  "#}
);

assert_html!(
  reversed_ordered_list,
  adoc! {r#"
    [%reversed]
    .Parts of an atom
    . Protons
    . Electrons
    . Neutrons
  "#},
  html! {r#"
    <section class="olist arabic"><h6 class="block-title">Parts of an atom</h6>
    <ol class="arabic" reversed><li>Protons</li><li>Electrons</li><li>Neutrons</li></ol></section>
  "#}
);

assert_html!(
  list_nested_ordered,
  adoc! {r#"
    . Step 1
    . Step 2
    .. Step 2a
    .. Step 2b
    . Step 3
  "#},
  html! {r#"
    <div class="olist arabic"><ol class="arabic"><li>Step 1</li><li>Step 2<ol class="loweralpha" type="a"><li>Step 2a</li><li>Step 2b</li></ol></li><li>Step 3</li></ol></div>
  "#}
);

assert_html!(
  list_unordered_within_ordered,
  adoc! {r#"
    . Linux
    * Fedora
    * Ubuntu
    * Slackware
    . BSD
    * FreeBSD
    * NetBSD
  "#},
  html! {r#"
    <div class="olist arabic"><ol class="arabic"><li>Linux<ul><li>Fedora</li><li>Ubuntu</li><li>Slackware</li></ul></li><li>BSD<ul><li>FreeBSD</li><li>NetBSD</li></ul></li></ol></div>
  "#}
);

assert_html!(
  list_ordered_marker_override,
  adoc! {r#"
    [lowerroman,start=5]
    . Five
    . Six
    [loweralpha]
    .. a
    .. b
    .. c
    . Seven
  "#},
  html! {r#"
    <div class="olist lowerroman"><ol class="lowerroman" type="i" start="5"><li>Five</li><li>Six<ol class="loweralpha" type="a"><li>a</li><li>b</li><li>c</li></ol></li><li>Seven</li></ol></div>
  "#}
);

assert_html!(
  checklist,
  adoc! {r#"
    [.custom-class]
    * [*] checked
    * [x] also checked
    * [ ] not checked
    * normal list item
  "#},
  html! {r#"
    <div class="ulist custom-class"><ul class="task-list"><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled checked> checked</li><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled checked> also checked</li><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled> not checked</li><li>normal list item</li></ul></div>
  "#}
);

assert_html!(
  list_interactive_checklist,
  adoc! {r#"
    [%interactive]
    * [*] checked
    * [x] also checked
    * [ ] not checked
  "#},
  html! {r#"
    <div class="ulist"><ul class="task-list"><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled checked> checked</li><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled checked> also checked</li><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" disabled> not checked</li></ul></div>
  "#}
);

assert_html!(
  ordered_list_not_checklist,
  adoc! {r#"
    . [*] checked
    . [ ] not checked
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li>[*] checked</li>
        <li>[ ] not checked</li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_item_continuation,
  adoc! {r#"
    * principle
    +
    with continuation
  "#},
  html! {r#"
    <div class="ulist"><ul><li><p>principle</p><p>with continuation</p></li></ul></div>
  "#}
);

assert_html!(
  list_item_2_continuations,
  adoc! {r#"
    * principle
    +
    with continuation
    +
    and another
  "#},
  html! {r#"
    <div class="ulist"><ul><li><p>principle</p><p>with continuation</p>
    <p>and another</p></li></ul></div>
  "#}
);

assert_html!(
  list_items_w_delimited_listing_blocks,
  adoc! {r#"
    * item 1
    +
    ----
    cont 1
    ----

    * item 2
    +
    ----
    cont 2
    ----
  "#},
  html! {r#"
    <div class="ulist"><ul><li><p>item 1</p><div class="listing-block"><pre>cont 1</pre></div></li><li><p>item 2</p><div class="listing-block"><pre>cont 2</pre></div></li></ul></div>
  "#}
);

assert_html!(
  list_items_w_delimited_blocks,
  adoc! {r#"
    * principle
    +
    --
    para 1

    para 2
    --

    * another item
    +
    --
    para 3

    para 4
    --
  "#},
  html! {r#"
    <div class="ulist"><ul><li><p>principle</p><div class="open-block"><div class="content"><p>para 1</p>
    <p>para 2</p></div></div></li><li><p>another item</p><div class="open-block"><div class="content"><p>para 3</p>
    <p>para 4</p></div></div></li></ul></div>
  "#}
);

assert_html!(
  list_empty_principle,
  adoc! {r#"
   . {empty}
   +
   --
   para
   --
 "#},
  html! {r#"
    <div class="olist arabic"><ol class="arabic"><li><p></p><div class="open-block"><div class="content"><p>para</p></div></div></li></ol></div>
   "#}
);

assert_html!(
  complex_continuation_example,
  adoc! {r#"
    * The header in AsciiDoc must start with a document title.
    +
    ----
    = Document Title
    ----
    +
    Keep in mind that the header is optional.

    * Optional author and revision information lines immediately follow the document title.
    +
    ----
    = Document Title
    Doc Writer <doc.writer@asciidoc.org>
    v1.0, 2022-01-01
    ----
  "#},
  html_e! {r#"
  <div class="ulist"><ul><li><p>The header in AsciiDoc must start with a document title.</p><div class="listing-block"><pre>= Document Title</pre></div><p>Keep in mind that the header is optional.</p></li><li><p>Optional author and revision information lines immediately follow the document title.</p><div class="listing-block"><pre>= Document Title
  Doc Writer &lt;doc.writer@asciidoc.org&gt;
  v1.0, 2022-01-01</pre></div></li></ul></div>"#}
);
assert_html!(
  bibliography_section_list,
  adoc! {r#"
    == Title

    A <<foo>> and a <<b123>>.

    Out of context [[[bib-ref]]] not recognized.

    [bibliography]
    == References

    * [[[foo]]] foo
    * [[[b123, 1]]] bar

    // break

    * no bib anchors
    * but should have bibliography class
  "#},
  html! {r##"
    <section class="doc-section level-1"><h2 id="_title">Title</h2><p>A <a href="#foo">[foo]</a> and a <a href="#b123">[1]</a>.</p>
    <p>Out of context [<a id="bib-ref" aria-hidden="true"></a>] not recognized.</p></section>
    <section class="doc-section level-1"><h2 id="_references">References</h2><div class="ulist bibliography"><ul class="bibliography"><li><a id="foo" aria-hidden="true"></a>[foo] foo</li><li><a id="b123" aria-hidden="true"></a>[1] bar</li></ul></div>
    <div class="ulist bibliography"><ul class="bibliography"><li>no bib anchors</li><li>but should have bibliography class</li></ul></div></section>
  "##}
);

assert_html!(
  // not documented, but dan said this is valid in zulip
  bibliography_list_not_in_section,
  adoc! {r#"
    [bibliography]
    - [[[taoup]]] Eric Steven Raymond. _The Art of Unix
      Programming_. Addison-Wesley. ISBN 0-13-142901-9.
    - [[[walsh-muellner]]] Norman Walsh & Leonard Muellner.
      _DocBook - The Definitive Guide_. O'Reilly & Associates. 1999.
      ISBN 1-56592-580-7.
  "#},
  html_e! {r#"
    <div class="ulist bibliography"><ul class="bibliography"><li><a id="taoup" aria-hidden="true"></a>[taoup] Eric Steven Raymond. <em>The Art of Unix
    Programming</em>. Addison-Wesley. ISBN 0-13-142901-9.</li><li><a id="walsh-muellner" aria-hidden="true"></a>[walsh-muellner] Norman Walsh &amp; Leonard Muellner.
    <em>DocBook - The Definitive Guide</em>. O&#8217;Reilly &amp; Associates. 1999.
    ISBN 1-56592-580-7.</li></ul></div>"#}
);
