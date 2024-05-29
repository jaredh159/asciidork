use asciidork_meta::JobSettings;
use test_utils::*;

mod helpers;

test_eval!(
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

test_eval!(
  xref_explicit_ids,
  adoc! {r#"
    [#custom]
    == Tigers

    Link to <<custom>>.

    Hashed link to <<#custom,Big CATS>> works too.

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
          <p>Hashed link to <a href="#custom">Big CATS</a> works too.</p>
        </div>
        <div class="paragraph">
          <p>Hashed macro to <a href="#custom">Tigers</a> works too.</p>
        </div>
      </div>
    </div>
  "##}
);

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
  legacy_inline_anchor_xrefs,
  adoc! {r#"
    [[step-1]]Download the software

    Refer to <<step-1>>.

    [[step-2,be sure to]]Lather, rinse, repeat

    Also, <<step-2>> do step 2.
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
  "##}
);
