use test_utils::*;

// NB: most of these tests are ported directly from the asciidoctor test suite
// @see https://gist.github.com/jaredh159/9e229fe1511aaea69e8f5658a8d1b5fd

assert_html!(
  asciidoctor_interdoc_xrefs_tests1,
  strict: false,
  adoc! {r#"
    // xref using angled bracket syntax with path sans extension
    <<tigers#>>

    // inter-document xref SHORTHAND syntax should assume
    // AsciiDoc extension if AsciiDoc extension not present
    - <<using-.net-web-services#,Using .NET web services>>
    - <<asciidoctor.1#,Asciidoctor Manual>>
    - <<path/to/document#,Document Title>>

    // xref macro with EXPLICIT inter-document target should assume
    // implicit AsciiDoc file extension if no file extension is present
    - xref:using-.net-web-services#[Using .NET web services]
    - xref:asciidoctor.1#[Asciidoctor Manual]
    - xref:document#[Document Title]
    - xref:path/to/document#[Document Title]
    - xref:include.d/document#[Document Title]

    // xref macro with implicit inter-document target should preserve path with file extension
    - xref:refcard.pdf[Refcard]
    - xref:asciidoctor.1[Asciidoctor Manual]
    - xref:sections.d/first[First Section]
  "#},
  html! {r##"
    <p><a href="tigers.html">tigers.html</a></p>
    <ul>
      <li><p><a href="using-.net-web-services.html">Using .NET web services</a></p></li>
      <li><p><a href="asciidoctor.1.html">Asciidoctor Manual</a></p></li>
      <li><p><a href="path/to/document.html">Document Title</a></p></li>
    </ul>
    <ul>
      <li><p><a href="using-.net-web-services">Using .NET web services</a></p></li>
      <li><p><a href="asciidoctor.1">Asciidoctor Manual</a></p></li>
      <li><p><a href="document.html">Document Title</a></p></li>
      <li><p><a href="path/to/document.html">Document Title</a></p></li>
      <li><p><a href="include.d/document.html">Document Title</a></p></li>
    </ul>
    <ul>
      <li><p><a href="refcard.pdf">Refcard</a></p></li>
      <li><p><a href="asciidoctor.1">Asciidoctor Manual</a></p></li>
      <li><p><a href="#sections.d/first">First Section</a></p></li>
    </ul>
  "##}
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests2,
  strict: false,
  adoc! {r#"
    // inter-document xref should only remove the file extension
    // part if the path contains a period elsewhere
    <<using-.net-web-services.adoc#,Using .NET web services>>

    // xref macro target containing dot should be interpreted as a path unless prefixed by #
    - xref:using-.net-web-services[Using .NET web services]
    - xref:#using-.net-web-services[Using .NET web services]

    // should not interpret double underscore in target of xref macro if sequence is preceded by a backslash
    xref:doc\__with_double__underscore.adoc[text]

    // should not interpret double underscore in target of xref shorthand if sequence is preceded by a backslash
    <<doc\__with_double__underscore#,text>>

    // xref using angled bracket syntax with ancestor path sans extension
    <<../tigers#,tigers>>

    // xref using angled bracket syntax with absolute path sans extension
    <</path/to/tigers#,tigers>>
  "#},
  html! {r##"
    <p><a href="using-.net-web-services.html">Using .NET web services</a></p>
    <ul>
      <li><p><a href="using-.net-web-services">Using .NET web services</a></p></li>
      <li><p><a href="#using-.net-web-services">Using .NET web services</a></p></li>
    </ul>
    <p><a href="doc__with_double__underscore.html">text</a></p>
    <p><a href="doc__with_double__underscore.html">text</a></p>
    <p><a href="../tigers.html">tigers</a></p>
    <p><a href="/path/to/tigers.html">tigers</a></p>
  "##}
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests3,
  strict: false,
  adoc! {r#"
    // xref using angled bracket syntax with path and extension
    <<tigers.adoc>>

    // xref using angled bracket syntax with path and extension with hash
    <<tigers.adoc#>>

    // xref using angled bracket syntax with path and extension with fragment
    <<tigers.adoc#id>>

    // xref using macro syntax with path and extension
    xref:tigers.adoc[]

    // xref using angled bracket syntax with path and fragment
    <<tigers#about>>

    // xref using angled bracket syntax with path, fragment and text
    <<tigers#about,About Tigers>>

    // xref using angled bracket syntax with path and custom relfilesuffix and outfilesuffix
    :relfileprefix: ../
    :outfilesuffix: /
    <<tigers#about,About Tigers>>

    // xref using angled bracket syntax with path and custom relfilesuffix
    :!relfileprefix:
    :!outfilesuffix:
    :relfilesuffix: /
    <<tigers#about,About Tigers>>
  "#},
  html! {r##"
    <p><a href="#tigers.adoc">[tigers.adoc]</a></p>
    <p><a href="tigers.html">tigers.html</a></p>
    <p><a href="tigers.html#id">tigers.html</a></p>
    <p><a href="tigers.html">tigers.html</a></p>
    <p><a href="tigers.html#about">tigers.html</a></p>
    <p><a href="tigers.html#about">About Tigers</a></p>
    <p><a href="../tigers/#about">About Tigers</a></p>
    <p><a href="tigers/#about">About Tigers</a></p>
  "##}
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests_include_1,
  resolving: b"info [#about]#tigers#.",
  adoc! {r#"
    include::tigers.adoc[]

    // xref using angled bracket syntax with path which has been included in this document
    <<tigers#about,About Tigers>>

    // explicit ref to included file with ext which was included (not in rx)
    <<tigers.adoc#about,About Tigers>>
  "#},
  html! {r##"
    <p>info <span id="about">tigers</span>.</p>
    <p><a href="#about">About Tigers</a></p>
    <p><a href="#about">About Tigers</a></p>
  "##}
);

assert_html!(
  issue_72_self_xrefs_from_included_files,
  resolving: bytes! {"
    == Section Baz

    See <<_section_baz, The Section about baz>>
  "},
  adoc! {r#"
    = Title

    include::include.adoc[]
  "#},
  contains: r##"See <a href="#_section_baz">The Section about baz</a></p>"##
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests_include_2,
  resolving: b"info [#about]#tigers#.",
  adoc! {r#"
    include::part1/tigers.adoc[]

    // xref using angled bracket syntax with nested path which has been included in this document
    <<part1/tigers#about,About Tigers>>

    // explicit ref to included file with ext which was included (not in rx)
    <<part1/tigers.adoc#about,About Tigers>>
  "#},
  html! {r##"
    <p>info <span id="about">tigers</span>.</p>
    <p><a href="#about">About Tigers</a></p>
    <p><a href="#about">About Tigers</a></p>
  "##}
);

assert_html!(
  interdoc_xref_resolves_link_text_from_include,
  resolving: OTHER_CHAPTERS,
  adoc! {r#"
    // should produce an internal anchor from an inter-document
    // xref to file included into current file

    [#ch1]
    == Chapter One

    So it begins.

    Read <<other-chapters.adoc#ch2>>.

    include::other-chapters.adoc[]
  "#},
  html! {r##"
    <section class="doc-section level-1">
      <h2 id="ch1">Chapter One</h2>
      <p>So it begins.</p>
      <p>Read <a href="#ch2">Chapter 2</a>.</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="ch2">Chapter 2</h2>
      <p>The plot thickens.</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="ch3">Chapter 3</h2>
      <p>The plot runs its course, predictably.</p>
    </section>
  "##}
);

assert_html!(
  interdoc_xref_include_all_tags,
  resolving: OTHER_CHAPTERS,
  adoc! {r#"
    [#ch1]
    == Chapter One

    // should produce an internal anchor for inter-document xref
    // to file included fully and partially
    Read <<other-chapters.adoc#ch2>>.

    include::other-chapters.adoc[tags=**]
  "#},
  contains: r##"<p>Read <a href="#ch2">Chapter 2</a>.</p>"##
);

assert_html!(
  interdoc_xref_include_partial,
  resolving: OTHER_CHAPTERS,
  adoc! {r#"
    [#ch1]
    == Chapter One

    // should not produce an internal anchor for inter-document
    // xref to file partially included into current file
    Read <<other-chapters.adoc#ch2,the next chapter>>.

    include::other-chapters.adoc[tags=ch2]
  "#},
  contains: r##"<p>Read <a href="#ch2">the next chapter</a>.</p>"##
);

assert_html!(
  doctitle_fallback_link_text,
  adoc! {r#"
    = The Document Title

    // should use doctitle as fallback link text if inter-document xref
    // points to current doc and no link text is provided
    See xref:test.adoc[]

    // should use doctitle of root document as fallback link text for
    // inter-document xref in AsciiDoc table cell that resolves to current doc
    |===
    a|From cell xref:test.adoc[]
    |===
  "#},
  contains:
    r##"<a href="#">The Document Title</a>"##,
    r##"From cell <a href="#">The Document Title</a>"##,
);

assert_html!(
  doctitle_refext_fallback_link_text,
  adoc! {r#"
    [reftext="Links and Stuff"]
    = The Document Title

    // should use reftext on document as fallback link text if inter-document
    // xref points to current doc and no link text is provided
    See xref:test.adoc[]

    // should use reftext on document as fallback link text if xref points
    // to empty fragment and no link text is provided
    See also xref:#[]
  "#},
  contains:
    r##"See <a href="#">Links and Stuff</a>"##,
    r##"See also <a href="#">Links and Stuff</a>"##
);

assert_html!(
  doctitle_refext_fallback_link_text_no_doc_header,
  adoc! {r#"
    // should use fallback link text if inter-document xref points
    // to current doc without header and no link text is provided
    See xref:test.adoc[]

    // should use fallback link text if fragment of
    // internal xref is empty and no link text is provided
    See also xref:#[]
  "#},
  contains:
    r##"See <a href="#">[^top]</a>"##,
    r##"See also <a href="#">[^top]</a>"##
);

// @see https://github.com/asciidoctor/asciidoctor/issues/3021
// @see https://github.com/asciidoctor/asciidoctor/issues/3231
assert_html!(
  asciidoctor_interdoc_xrefs_edge_cases_from_gh_issues,
  strict: false,
  adoc! {r#"
    - <<a#>>
    - <<b.adoc#>>
    - <<c.1#>>
    - <<d.1.adoc#>>

    - xref:e#[]
    - xref:f[]
    - xref:g.adoc#[]
    - xref:h.adoc[]
    - xref:i.1#[]
    - xref:j.1[]
    - xref:k.pdf#[]
    - xref:l.pdf[]
    - xref:m-using-.net-services[]
    - xref:#n-using-.net-services[]
  "#},
  html! {r##"
    <ul>
      <li><p><a href="a.html">a.html</a></p></li>
      <li><p><a href="b.html">b.html</a></p></li>
      <li><p><a href="c.1.html">c.1.html</a></p></li>
      <li><p><a href="d.1.html">d.1.html</a></p></li>
      <li><p><a href="e.html">e.html</a></p></li>
      <li><p><a href="#f">[f]</a></p></li>
      <li><p><a href="g.html">g.html</a></p></li>
      <li><p><a href="h.html">h.html</a></p></li>
      <li><p><a href="i.1">i.1</a></p></li>
      <li><p><a href="j.1">j.1</a></p></li>
      <li><p><a href="k.pdf">k.pdf</a></p></li>
      <li><p><a href="l.pdf">l.pdf</a></p></li>
      <li><p><a href="m-using-.net-services">m-using-.net-services</a></p></li>
      <li><p><a href="#n-using-.net-services">[n-using-.net-services]</a></p></li>
    </ul>
  "##}
);

const OTHER_CHAPTERS: &[u8] = b"// tag::ch2[]
[#ch2]
// tag::ch2-noid[]
== Chapter 2

The plot thickens.
// end::ch2-noid[]
// end::ch2[]

[#ch3]
== Chapter 3

The plot runs its course, predictably.
";
