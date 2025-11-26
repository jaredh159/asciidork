use test_utils::*;

assert_html!(
  xrefs,
  strict: false,
  adoc! {r#"
    == Tigers

    See <<_tigers>> for more information.

    This <<_ligers>> xref is broken.
  "#},
  html! {r##"
    <section class="doc-section level-1"><h2 id="_tigers">Tigers</h2><p>See <a href="#_tigers">Tigers</a> for more information.</p>
    <p>This <a href="#_ligers">[_ligers]</a> xref is broken.</p></section>
  "##}
);

assert_html!(
  xref_explicit_ids,
  adoc! {r#"
    [#custom]
    == Tigers

    Link to <<custom>>.

    Reftext link to <<custom,Big CATS>> works too.

    Hashed reftext link to <<#custom,Big CATS>> works too.

    Quoted reftext link to <<#custom,"Big CATS">> works too.

    Empty reftext to <<custom,>> works too.

    Hashed macro to xref:#custom[] works too.
  "#},
  html! {r##"
    <section class="doc-section level-1"><h2 id="custom">Tigers</h2><p>Link to <a href="#custom">Tigers</a>.</p>
    <p>Reftext link to <a href="#custom">Big CATS</a> works too.</p>
    <p>Hashed reftext link to <a href="#custom">Big CATS</a> works too.</p>
    <p>Quoted reftext link to <a href="#custom">"Big CATS"</a> works too.</p>
    <p>Empty reftext to <a href="#custom">Tigers</a> works too.</p>
    <p>Hashed macro to <a href="#custom">Tigers</a> works too.</p></section>
  "##}
);

assert_html!(
  xref_custom_reftext,
  adoc! {r#"
    [reftext=Big _cats!_]
    == Tigers

    Link to <<_tigers>>.
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="_tigers">Tigers</h2>
      <p>Link to <a href="#_tigers">Big <em>cats!</em></a>.</p>
    </section>
  "##}
);

assert_html!(
  xref_explicit_link_text_empty,
  adoc! {r#"
    == Tigers

    Link to <<_tigers,>>.
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="_tigers">Tigers</h2>
      <p>Link to <a href="#_tigers">Tigers</a>.</p>
    </section>
  "##}
);

assert_html!(
  xref_macro,
  adoc! {r#"
    [#tigers]
    == Tigers

    Link to xref:tigers[].

    Link xref:tigers[with target].
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="tigers">Tigers</h2>
      <p>Link to <a href="#tigers">Tigers</a>.</p>
      <p>Link <a href="#tigers">with target</a>.</p>
    </section>
  "##}
);

assert_html!(
  xref_complex_linktext,
  adoc! {r#"
    == Tigers

    Link to <<_tigers,`+[tigers]+`>>.
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="_tigers">Tigers</h2>
      <p>Link to <a href="#_tigers"><code>[tigers]</code></a>.</p>
    </section>
  "##}
);

assert_html!(
  xref_quoted_linktext,
  adoc! {r#"
    == Tigers

    Link to <<_tigers,"Big Cats">>.

    Link to xref:_tigers["Big Cats"].
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="_tigers">Tigers</h2>
      <p>Link to <a href="#_tigers">"Big Cats"</a>.</p>
      <p>Link to <a href="#_tigers">"Big Cats"</a>.</p>
    </section>
  "##}
);

assert_html!(
  xref_escaped_bracket_in_linktext,
  adoc! {r#"
    xref:tigers[[tigers\] are cats]

    [#tigers]
    == Tigers
  "#},
  html! {r##"
    <p><a href="#tigers">[tigers] are cats</a></p>
    <section class="doc-section level-1">
      <h2 id="tigers">Tigers</h2>
    </section>
  "##}
);

assert_html!(
  xref_to_text_span,
  adoc! {r#"
    Here is [#tigers]#a text span#.

    And a <<tigers>> link.
  "#},
  html! {r##"
    <p>Here is <span id="tigers">a text span</span>.</p>
    <p>And a <a href="#tigers">a text span</a> link.</p>
  "##}
);

assert_html!(
  inline_anchor_xrefs,
  adoc! {r#"
    [[step-1]]Download the software

    Refer to <<step-1>>.

    [[step-2,be sure to]]Lather, rinse, repeat

    Also, <<step-2>> do step 2.

    anchor:step-3[Done]Finished

    You're <<step-3>>!
  "#},
  html! {r##"
    <p><a id="step-1" aria-hidden="true"></a>Download the software</p>
    <p>Refer to <a href="#step-1">[step-1]</a>.</p>
    <p><a id="step-2" aria-hidden="true"></a>Lather, rinse, repeat</p>
    <p>Also, <a href="#step-2">be sure to</a> do step 2.</p>
    <p><a id="step-3" aria-hidden="true"></a>Finished</p>
    <p>You&#8217;re <a href="#step-3">Done</a>!</p>
  "##}
);

// asciidoctor/test/links_test.rb
assert_html!(
  asciidoctor_xrefs_test_rb1,
  adoc! {r#"
    // inline ref
    Foo.[[tigers1]] bar.anchor:tigers2[]

    // escaped inline ref
    Foo.\[[tigers1]] bar.\anchor:tigers2[]

    // inline ref can start with colon
    [[:idname]] text

    // inline ref cannot start with digit
    [[1-install]] text

    // reftext of macro inline ref can resolve to empty
    anchor:id-only[{empty}]text

    // inline ref with reftext
    [[tigers3,Tigers]] anchor:tigers4[Tigers]
  "#},
  html! {r##"
    <p>Foo.<a id="tigers1" aria-hidden="true"></a> bar.<a id="tigers2" aria-hidden="true"></a></p>
    <p>Foo.[[tigers1]] bar.anchor:tigers2[]</p>
    <p><a id=":idname" aria-hidden="true"></a> text</p>
    <p>[[1-install]] text</p>
    <p><a id="id-only" aria-hidden="true"></a>text</p>
    <p><a id="tigers3" aria-hidden="true"></a> <a id="tigers4" aria-hidden="true"></a></p>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb2,
  strict: false,
  adoc! {r#"
    :label-tigers: Tigers

    // should substitute attribute references in reftext when registering inline ref
    [[tigers4,{label-tigers}]] anchor:tigers5[{label-tigers}]
    <<tigers4>> <<tigers5>>

    // repeating inline anchor macro with empty reftext
    anchor:one[] anchor:two[] anchor:three[]

    // mixed inline anchor macro and anchor shorthand with empty reftext
    anchor:one[][[two]]anchor:three[][[four]]anchor:five[]

    // unescapes square bracket in reftext of anchor macro
    see <<foo>> anchor:foo[b[a\]r]tex

    // xref using angled bracket syntax
    <<not-found>>

    // xref using angled bracket syntax with explicit hash
    <<#not-found2>>
  "#},
  raw_html! {r##"
    <p><a id="tigers4" aria-hidden="true"></a> <a id="tigers5" aria-hidden="true"></a>
    <a href="#tigers4">Tigers</a> <a href="#tigers5">Tigers</a></p><p><a id="one" aria-hidden="true"></a> <a id="two" aria-hidden="true"></a> <a id="three" aria-hidden="true"></a></p><p><a id="one" aria-hidden="true"></a><a id="two" aria-hidden="true"></a><a id="three" aria-hidden="true"></a><a id="four" aria-hidden="true"></a><a id="five" aria-hidden="true"></a></p><p>see <a href="#foo">b[a]r</a> <a id="foo" aria-hidden="true"></a>tex</p><p><a href="#not-found">[not-found]</a></p><p><a href="#not-found2">[not-found2]</a></p>"##}
);

assert_html!(
  asciidoctor_xrefs_test_rb3,
  strict: false,
  adoc! {r#"
    // xref with target that begins with attribute reference in title (1/2)
    :lessonsdir: lessons

    [#lesson-1-listing]
    == <<{lessonsdir}/lesson-1#,Lesson 1>>

    A summary of the first lesson.

    // xref with target that begins with attribute reference in title (2/2)

    [#lesson-2-listing]
    == xref:{lessonsdir}/lesson-2.adoc[Lesson 2]

    A summary of the second lesson.

    == rest

    // xref using angled bracket syntax inline with text
    Want to learn <<tigers,about tigers>>?

    // xref with escaped text
    See the <<tigers, `+[tigers]+`>> for more.
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="lesson-1-listing"><a href="lessons/lesson-1.html">Lesson 1</a></h2>
      <p>A summary of the first lesson.</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="lesson-2-listing"><a href="lessons/lesson-2.html">Lesson 2</a></h2>
      <p>A summary of the second lesson.</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="_rest">rest</h2>
      <p>Want to learn <a href="#tigers">about tigers</a>?</p>
      <p>See the <a href="#tigers"><code>[tigers]</code></a> for more.</p>
    </section>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb4,
  strict: false,
  adoc! {r#"
    // multiple xref macros with implicit text in single line
    This document has two sections, xref:sect-a[] and xref:sect-b[].

    // xref using macro syntax with explicit hash
    xref:#tigers[]

    // xref using macro syntax with label
    xref:tigers[About Tigers]

    // xref using macro syntax inline with text
    Want to learn xref:tigers[about tigers]?

    // xref using macro syntax with text that ends with an escaped closing bracket
    xref:tigers[[foobar\]]

    // xref using macro syntax with text that contains an escaped closing bracket
    xref:tigers[[tigers\] are cats]

    // unescapes square bracket in reftext used by xref
    anchor:foo[b[a\]r]about

    // xref using invalid macro syntax does not create link
    xref:tigers
  "#},
  html! {r##"
    <p>This document has two sections, <a href="#sect-a">[sect-a]</a> and <a href="#sect-b">[sect-b]</a>.</p>
    <p><a href="#tigers">[tigers]</a></p>
    <p><a href="#tigers">About Tigers</a></p>
    <p>Want to learn <a href="#tigers">about tigers</a>?</p>
    <p><a href="#tigers">[foobar]</a></p>
    <p><a href="#tigers">[tigers] are cats</a></p>
    <p><a id="foo" aria-hidden="true"></a>about</p>
    <p>xref:tigers</p>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb5,
  adoc! {r#"
    // anchor creates reference
    [[tigers]]Tigers roam here.

    See <<tigers>>.

    // anchor with label creates reference
    [[tigers2,Tigers]]Tigers roam here.

    See <<tigers2>>.

    // anchor with quoted label creates reference with quoted label text
    [[tigers3,"Tigers roam here"]]Tigers roam here.

    See <<tigers3>>.

    // anchor with label containing a comma creates reference
    [[tigers4,Tigers, scary tigers, roam here]]Tigers roam here.

    See <<tigers4>>.
  "#},
  contains:
    r##"See <a href="#tigers">[tigers]</a>."##,
    r##"See <a href="#tigers2">Tigers</a>."##,
    r##"See <a href="#tigers3">"Tigers roam here"</a>."##,
    r##"See <a href="#tigers4">Tigers, scary tigers, roam here</a>."##,
);

assert_html!(
  xref_labels_forward_backward,
  adoc! {r#"
    // xref uses title of target as label for forward and backward references in html output
    == Section A

    <<_section_b>>

    == Section B

    <<_section_a>>
  "#},
  contains:
    r##"<h2 id="_section_a">Section A</h2>"##,
    r##"<a href="#_section_a">Section A</a>"##,
    r##"<h2 id="_section_b">Section B</h2>"##,
    r##"<a href="#_section_b">Section B</a>"##
);
