use test_utils::*;

assert_html!(
  footnote,
  "foo.footnote:[bar _baz_]",
  html! {r##"
    <p>foo.<a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></p><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">bar <em>baz</em> <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section>
  "##}
);

assert_html!(
  footnote_ref_prev,
  adoc! {r#"
    foo footnote:thing[bar] baz.

    baz footnote:thing[] qux.
  "#},
  html! {r##"
    <p>foo <a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a> baz.</p>
    <p>baz <a class="footnote-ref" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a> qux.</p><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">bar <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section>
  "##}
);

assert_html!(
  two_footnotes_w_cust,
  adoc! {r#"
    foo.footnote:[bar _baz_]

    lol.footnote:cust[baz]
  "#},
  html! {r##"
    <p>foo.<a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></p>
    <p>lol.<a class="footnote-ref" id="_footnoteref_2" href="#_footnote_2" title="View footnote 2" role="doc-noteref">[2]</a></p><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">bar <em>baz</em> <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li><li class="footnote" id="_footnote_2" role="doc-endnote">baz <a class="footnote-backref" href="#_footnoteref_2" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section>
  "##}
);

// tests descending into new cell document, maintaining global footnotes
// and also that we don't render the footnote div twice
// i think jirutka gets it wrong here
// assert_html!(
//   adoc_cell_footnotes,
//   adoc! {r#"
//     main footnote:[main note 1]
//
//     |===
//     a|AsciiDoc footnote:[cell note]
//     |===
//
//     main footnote:[main note 2]
//   "#},
//   html! { r##"
//     <p>main <a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></p>
//     <div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top"><p>AsciiDoc <a class="footnote-ref" id="_footnoteref_2" href="#_footnote_2" title="View footnote 2" role="doc-noteref">[2]</a></p><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_2" role="doc-endnote">cell note <a class="footnote-backref" href="#_footnoteref_2" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section></td></tr></tbody></table></div>
//     <p>main <a class="footnote-ref" id="_footnoteref_3" href="#_footnote_3" title="View footnote 3" role="doc-noteref">[3]</a></p><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">main note 1 <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li><li class="footnote" id="_footnote_3" role="doc-endnote">main note 2 <a class="footnote-backref" href="#_footnoteref_3" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section>
//   "##}
// );
