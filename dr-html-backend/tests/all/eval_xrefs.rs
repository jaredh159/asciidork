use asciidork_core::JobSettings;
use test_utils::*;

// NB: many of these tests are ported directly from the asciidoctor test suite
// @see https://gist.github.com/jaredh159/9e229fe1511aaea69e8f5658a8d1b5fd

assert_html!(
  xrefs,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    == Tigers

    See <<_tigers>> for more information.

    This <<_ligers>> xref is broken.
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>See <a href="#_tigers">Tigers</a> for more information.</p>
        </div>
        <div class="paragraph">
          <p>This <a href="#_ligers">[_ligers]</a> xref is broken.</p>
        </div>
      </div>
    </div>
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
    <div class="sect1">
      <h2 id="custom">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#custom">Tigers</a>.</p>
        </div>
        <div class="paragraph">
          <p>Reftext link to <a href="#custom">Big CATS</a> works too.</p>
        </div>
        <div class="paragraph">
          <p>Hashed reftext link to <a href="#custom">Big CATS</a> works too.</p>
        </div>
        <div class="paragraph">
          <p>Quoted reftext link to <a href="#custom">"Big CATS"</a> works too.</p>
        </div>
        <div class="paragraph">
          <p>Empty reftext to <a href="#custom">Tigers</a> works too.</p>
        </div>
        <div class="paragraph">
          <p>Hashed macro to <a href="#custom">Tigers</a> works too.</p>
        </div>
      </div>
    </div>
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
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#_tigers">Big <em>cats!</em></a>.</p>
        </div>
      </div>
    </div>
  "##}
);

assert_html!(
  xref_explicit_link_text_empty,
  adoc! {r#"
    == Tigers

    Link to <<_tigers,>>.
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#_tigers">Tigers</a>.</p>
        </div>
      </div>
    </div>
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
    <div class="sect1">
      <h2 id="tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#tigers">Tigers</a>.</p>
        </div>
        <div class="paragraph">
          <p>Link <a href="#tigers">with target</a>.</p>
        </div>
      </div>
    </div>
  "##}
);

assert_html!(
  xref_complex_linktext,
  adoc! {r#"
    == Tigers

    Link to <<_tigers,`+[tigers]+`>>.
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#_tigers"><code>[tigers]</code></a>.</p>
        </div>
      </div>
    </div>
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
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Link to <a href="#_tigers">"Big Cats"</a>.</p>
        </div>
        <div class="paragraph">
          <p>Link to <a href="#_tigers">"Big Cats"</a>.</p>
        </div>
      </div>
    </div>
  "##}
);

assert_html!(
  xref_escraped_bracket_in_linktext,
  adoc! {r#"
    xref:tigers[[tigers\] are cats]

    [#tigers]
    == Tigers
  "#},
  html! {r##"
    <div id="preamble">
      <div class="sectionbody">
        <div class="paragraph">
          <p><a href="#tigers">[tigers] are cats</a></p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="tigers">Tigers</h2>
      <div class="sectionbody"></div>
    </div>
  "##}
);

assert_html!(
  xref_to_text_span,
  adoc! {r#"
    Here is [#tigers]#a text span#.

    And a <<tigers>> link.
  "#},
  html! {r##"
    <div class="paragraph">
      <p>Here is <span id="tigers">a text span</span>.</p>
    </div>
    <div class="paragraph">
      <p>And a <a href="#tigers">a text span</a> link.</p>
    </div>
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
    <div class="paragraph">
      <p><a id="step-1"></a>Download the software</p>
    </div>
    <div class="paragraph">
      <p>Refer to <a href="#step-1">[step-1]</a>.</p>
    </div>
    <div class="paragraph">
      <p><a id="step-2"></a>Lather, rinse, repeat</p>
    </div>
    <div class="paragraph">
      <p>Also, <a href="#step-2">be sure to</a> do step 2.</p>
    </div>
    <div class="paragraph">
      <p><a id="step-3"></a>Finished</p>
    </div>
    <div class="paragraph">
      <p>You&#8217;re <a href="#step-3">Done</a>!</p>
    </div>
  "##}
);

assert_html!(
  inline_anchor_starting_cell,
  adoc! {r#"
    The highest peak in the Front Range is <<grays-peak>>, which tops <<mount-evans>> by just a few feet.

    [cols="1s,1"]
    |===
    |[[mount-evans,Mount Evans]]Mount Evans
    |14,271 feet

    h|[[grays-peak,Grays Peak]]
    Grays Peak
    |14,278 feet
    |===
  "#},
  html! {r##"
    <div class="paragraph">
      <p>The highest peak in the Front Range is <a href="#grays-peak">Grays Peak</a>, which tops <a href="#mount-evans">Mount Evans</a> by just a few feet.</p>
    </div>
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 50%;"><col style="width: 50%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock"><strong><a id="mount-evans"></a>Mount Evans</strong></p>
          </td>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">14,271 feet</p>
          </td>
        </tr>
        <tr>
          <th class="tableblock halign-left valign-top">
            <p class="tableblock"><a id="grays-peak"></a> Grays Peak</p>
          </th>
          <td class="tableblock halign-left valign-top">
            <p class="tableblock">14,278 feet</p>
          </td>
        </tr>
      </tbody>
    </table>
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
    <div class="paragraph">
      <p>Foo.<a id="tigers1"></a> bar.<a id="tigers2"></a></p>
    </div>
    <div class="paragraph">
      <p>Foo.[[tigers1]] bar.anchor:tigers2[]</p>
    </div>
    <div class="paragraph">
      <p><a id=":idname"></a> text</p>
    </div>
    <div class="paragraph">
      <p>[[1-install]] text</p>
    </div>
    <div class="paragraph">
      <p><a id="id-only"></a>text</p>
    </div>
    <div class="paragraph">
      <p><a id="tigers3"></a> <a id="tigers4"></a></p>
    </div>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb2,
  |s: &mut JobSettings| s.strict = false,
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
  html! {r##"
    <div class="paragraph">
      <p><a id="tigers4"></a> <a id="tigers5"></a> <a href="#tigers4">Tigers</a> <a href="#tigers5">Tigers</a></p>
    </div>
    <div class="paragraph">
      <p><a id="one"></a> <a id="two"></a> <a id="three"></a></p>
    </div>
    <div class="paragraph">
      <p><a id="one"></a><a id="two"></a><a id="three"></a><a id="four"></a><a id="five"></a></p>
    </div>
    <div class="paragraph">
      <p>see <a href="#foo">b[a]r</a> <a id="foo"></a>tex</p>
    </div>
    <div class="paragraph">
      <p><a href="#not-found">[not-found]</a></p>
    </div>
    <div class="paragraph">
      <p><a href="#not-found2">[not-found2]</a></p>
    </div>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb3,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="sect1">
      <h2 id="lesson-1-listing"><a href="lessons/lesson-1.html">Lesson 1</a></h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>A summary of the first lesson.</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="lesson-2-listing"><a href="lessons/lesson-2.html">Lesson 2</a></h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>A summary of the second lesson.</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_rest">rest</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Want to learn <a href="#tigers">about tigers</a>?</p>
        </div>
        <div class="paragraph">
          <p>See the <a href="#tigers"><code>[tigers]</code></a> for more.</p>
        </div>
      </div>
    </div>
  "##}
);

assert_html!(
  asciidoctor_xrefs_test_rb4,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="paragraph">
      <p>This document has two sections, <a href="#sect-a">[sect-a]</a> and <a href="#sect-b">[sect-b]</a>.</p>
    </div>
    <div class="paragraph"><p><a href="#tigers">[tigers]</a></p></div>
    <div class="paragraph"><p><a href="#tigers">About Tigers</a></p></div>
    <div class="paragraph"><p>Want to learn <a href="#tigers">about tigers</a>?</p></div>
    <div class="paragraph"><p><a href="#tigers">[foobar]</a></p></div>
    <div class="paragraph"><p><a href="#tigers">[tigers] are cats</a></p></div>
    <div class="paragraph"><p><a id="foo"></a>about</p></div>
    <div class="paragraph"><p>xref:tigers</p></div>
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

assert_html!(
  xref_as_title_and_other_weird_places,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    == Intro

    // should resolve forward xref in title of block with ID
    [#p1]
    .<<conclusion>>
    paragraph text

    [#conclusion]
    == Conclusion

    // should not fail to resolve broken xref in title of block with ID
    [#p2]
    .<<DNE>>
    paragraph text
  "#},
  contains:
    r##"<div class="title"><a href="#conclusion">Conclusion</a></div>"##,
    r##"<div class="title"><a href="#DNE">[DNE]</a></div>"##
);

// NB: we generate the id for section 2 different from asciidoctor
assert_html!(
  resolves_broken_xref_in_title,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    [#s1]
    == <<DNE>>

    // should not fail to resolve broken xref in section title
    == <<s1>>
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="s1"><a href="#DNE">[DNE]</a></h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_s1"><a href="#s1">[DNE]</a></h2>
      <div class="sectionbody"></div>
    </div>
  "##}
);

assert_html!(
  breaks_circular_xref,
  adoc! {r#"
    [#a]
    == A <<b>>

    // should break circular xref reference in section title
    [#b]
    == B <<a>>
  "#},
  // html doesn't matter, only that we don't stack overflow
  contains: r##"<a href="#b">B [a]</a>"##,
);

assert_html!(
  drops_nested_anchor,
  adoc! {r#"
    [#a]
    == See <<b>>

    // should drop nested anchor in xreftext
    [#b]
    == Consult https://google.com[Google]
  "#},
  contains: r##"<h2 id="a">See <a href="#b">Consult Google</a></h2>"##,
);
