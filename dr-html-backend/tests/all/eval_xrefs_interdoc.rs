use asciidork_meta::JobSettings;
use test_utils::*;

// ported from asciidoctor/test/links_test.rb
assert_html!(
  asciidoctor_interdoc_xrefs_tests1,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="paragraph">
      <p><a href="tigers.html">tigers.html</a></p>
    </div>
    <div class="ulist">
      <ul>
        <li><p><a href="using-.net-web-services.html">Using .NET web services</a></p></li>
        <li><p><a href="asciidoctor.1.html">Asciidoctor Manual</a></p></li>
        <li><p><a href="path/to/document.html">Document Title</a></p></li>
      </ul>
    </div>
    <div class="ulist">
      <ul>
        <li><p><a href="using-.net-web-services">Using .NET web services</a></p></li>
        <li><p><a href="asciidoctor.1">Asciidoctor Manual</a></p></li>
        <li><p><a href="document.html">Document Title</a></p></li>
        <li><p><a href="path/to/document.html">Document Title</a></p></li>
        <li><p><a href="include.d/document.html">Document Title</a></p></li>
      </ul>
    </div>
    <div class="ulist">
      <ul>
        <li><p><a href="refcard.pdf">Refcard</a></p></li>
        <li><p><a href="asciidoctor.1">Asciidoctor Manual</a></p></li>
        <li><p><a href="#sections.d/first">First Section</a></p></li>
      </ul>
    </div>
  "##}
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests2,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="paragraph">
      <p><a href="using-.net-web-services.html">Using .NET web services</a></p>
    </div>
    <div class="ulist">
      <ul>
        <li><p><a href="using-.net-web-services">Using .NET web services</a></p></li>
        <li><p><a href="#using-.net-web-services">Using .NET web services</a></p></li>
      </ul>
    </div>
    <div class="paragraph">
      <p><a href="doc__with_double__underscore.html">text</a></p>
    </div>
    <div class="paragraph">
      <p><a href="doc__with_double__underscore.html">text</a></p>
    </div>
    <div class="paragraph">
      <p><a href="../tigers.html">tigers</a></p>
    </div>
    <div class="paragraph">
      <p><a href="/path/to/tigers.html">tigers</a></p>
    </div>
  "##}
);

assert_html!(
  asciidoctor_interdoc_xrefs_tests3,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="paragraph">
      <p><a href="#tigers.adoc">[tigers.adoc]</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers.html">tigers.html</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers.html#id">tigers.html</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers.html">tigers.html</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers.html#about">tigers.html</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers.html#about">About Tigers</a></p>
    </div>
    <div class="paragraph">
      <p><a href="../tigers/#about">About Tigers</a></p>
    </div>
    <div class="paragraph">
      <p><a href="tigers/#about">About Tigers</a></p>
    </div>
  "##}
);

// @see https://github.com/asciidoctor/asciidoctor/issues/3021
// @see https://github.com/asciidoctor/asciidoctor/issues/3231
assert_html!(
  asciidoctor_interdoc_xrefs_edge_cases_from_gh_issues,
  |s: &mut JobSettings| s.strict = false,
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
    <div class="ulist">
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
    </div>
  "##}
);
