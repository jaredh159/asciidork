use test_utils::*;

assert_html!(
  footnote,
  "foo.footnote:[bar _baz_]",
  html! {r##"
    <div class="paragraph">
      <p>foo.
        <sup class="footnote">
          [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
        </sup>
      </p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
      </div>
    </div>
  "##}
);

assert_html!(
  footnote_ref_prev,
  adoc! {r#"
    foo footnote:thing[bar] baz.

    baz footnote:thing[] qux.
  "#},
  html! {r##"
    <div class="paragraph">
      <p>foo <sup class="footnote" id="_footnote_thing">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup> baz.</p>
    </div>
    <div class="paragraph">
      <p>baz <sup class="footnoteref">[<a class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup> qux.</p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. bar
      </div>
    </div>
  "##}
);

assert_html!(
  duplicate_content_externalized_footnote,
  adoc! {r#"
    :fn-foo: footnote:thing[foo]

    one.{fn-foo}

    two.{fn-foo}
"#},
  html! {r##"
    <div class="paragraph">
      <p>
        one.<sup class="footnote" id="_footnote_thing">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup>
      </p>
    </div>
    <div class="paragraph">
      <p>
        two.<sup class="footnoteref">[<a class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup>
      </p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. foo
      </div>
    </div>
  "##}
);

assert_html!(
  two_footnotes_w_cust,
  adoc! {r#"
    foo.footnote:[bar _baz_]

    lol.footnote:cust[baz]
  "#},
  html! {r##"
    <div class="paragraph">
      <p>foo.
        <sup class="footnote">
          [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
        </sup>
      </p>
    </div>
    <div class="paragraph">
      <p>lol.
        <sup class="footnote" id="_footnote_cust">
          [<a id="_footnoteref_2" class="footnote" href="#_footnotedef_2" title="View footnote.">2</a>]
        </sup>
      </p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1">
        <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
      </div>
      <div class="footnote" id="_footnotedef_2">
        <a href="#_footnoteref_2">2</a>. baz
      </div>
    </div>
  "##}
);

// tests descending into new cell document, maintaining global footnotes
// and also that we don't render the footnote div twice
assert_html!(
  adoc_cell_footnotes,
  adoc! {r#"
    main footnote:[main note 1]

    |===
    a|AsciiDoc footnote:[cell note]
    |===

    main footnote:[main note 2]
  "#},
  html! { r##"
    <div class="paragraph">
      <p>main <sup class="footnote">[<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]</sup></p>
    </div>
    <table class="tableblock frame-all grid-all stretch">
      <colgroup><col style="width: 100%;"></colgroup>
      <tbody>
        <tr>
          <td class="tableblock halign-left valign-top">
            <div class="content">
              <div class="paragraph">
                <p>AsciiDoc <sup class="footnote">[<a id="_footnoteref_2" class="footnote" href="#_footnotedef_2" title="View footnote.">2</a>]</sup></p>
              </div>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
    <div class="paragraph">
      <p>main <sup class="footnote">[<a id="_footnoteref_3" class="footnote" href="#_footnotedef_3" title="View footnote.">3</a>]</sup></p>
    </div>
    <div id="footnotes">
      <hr>
      <div class="footnote" id="_footnotedef_1"><a href="#_footnoteref_1">1</a>. main note 1</div>
      <div class="footnote" id="_footnotedef_2"><a href="#_footnoteref_2">2</a>. cell note</div>
      <div class="footnote" id="_footnotedef_3"><a href="#_footnoteref_3">3</a>. main note 2</div>
    </div>
  "##}
);
