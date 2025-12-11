use asciidork_parser::prelude::*;
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
      <pre class="highlight">
        <code class="language-bash" data-lang="bash">cowsay hi</code>
      </pre>
    </figure>
  "#}
);

assert_html!(
  linenums,
  adoc! {r#"
    ``` javascript, numbered
    alert("Hello, World!")
    ```
  "#},
  html! {r##"
    <div class="listing-block">
      <pre class="highlight linenums">
        <code class="language-javascript" data-lang="javascript">alert("Hello, World!")</code>
      </pre>
    </div>
  "##}
);

assert_html!(
  example,
  adoc! {r#"
    // .collapsible
    .Toggle *Me*
    [%collapsible]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-open
    .Toggle Me
    [%collapsible%open]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-with-id-and-role
    .Toggle Me
    [#lorem.ipsum%collapsible]
    ====
    This content is revealed when the user clicks the words "Toggle Me".
    ====

    // .collapsible-without-title
    [%collapsible]
    ====
    This content is revealed when the user clicks the words "Details".
    ====
  "#},
  html! {r##"
    <details>
      <summary>Toggle <strong>Me</strong></summary>
      <div class="content">
        <p>This content is revealed when the user clicks the words "Toggle Me".</p>
      </div>
    </details>
    <details open>
      <summary>Toggle Me</summary>
      <div class="content">
        <p>This content is revealed when the user clicks the words "Toggle Me".</p>
      </div>
    </details>
    <details id="lorem" class="ipsum">
      <summary>Toggle Me</summary>
      <div class="content">
        <p>This content is revealed when the user clicks the words "Toggle Me".</p>
      </div>
    </details>
    <details>
      <div class="content">
        <p>This content is revealed when the user clicks the words "Details".</p>
      </div>
    </details>
  "##}
);

assert_html!(
  image,
  adoc! {r#"
    // .with-link-and-window-blank
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", window=_blank]

    // .with-link-and-noopener
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", opts=noopener]

    // .with-link-and-nofollow
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655", opts=nofollow]

    // .with-link-self
    image::sunset.jpg[link=self]

    // .with-link-none
    image::sunset.jpg[link=none]

    // .with-loading-lazy
    image::sunset.jpg[loading=lazy]

    // .html5s-image-default-link-self
    :html5s-image-default-link: self
    image::sunset.jpg[]

    // .html5s-image-default-link-self-with-link-none
    :html5s-image-default-link: self
    image::sunset.jpg[link=none]

    // .html5s-image-default-link-self-with-link-url
    :html5s-image-default-link: self
    image::sunset.jpg[link="http://www.flickr.com/photos/javh/5448336655"]
  "#},
  html! {r##"
    <div class="image-block">
      <a class="image" href="http://www.flickr.com/photos/javh/5448336655" target="_blank" rel="noopener">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
    <div class="image-block">
      <a class="image" href="http://www.flickr.com/photos/javh/5448336655" rel="noopener">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
    <div class="image-block">
      <a class="image" href="http://www.flickr.com/photos/javh/5448336655" rel="nofollow">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
    <div class="image-block">
      <a class="image bare" href="sunset.jpg" title="Open the image in full size" aria-label="Open the image in full size">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
    <div class="image-block">
      <img src="sunset.jpg" alt="sunset">
    </div>
    <div class="image-block">
      <img src="sunset.jpg" alt="sunset" loading="lazy">
    </div>
    <div class="image-block">
      <a class="image bare" href="sunset.jpg" title="Open the image in full size" aria-label="Open the image in full size">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
    <div class="image-block">
      <img src="sunset.jpg" alt="sunset">
    </div>
    <div class="image-block">
      <a class="image" href="http://www.flickr.com/photos/javh/5448336655">
        <img src="sunset.jpg" alt="sunset">
      </a>
    </div>
  "##}
);

assert_html!(
  inline_image,
  adoc! {r#"
    // .image-with-link-and-window-blank
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", window=_blank]

    // .image-with-link-and-noopener
    // NB: jirutka adds rel="noopener" but we don't because target=_blank is not used
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", opts=noopener]

    // .with-link-and-nofollow
    image:linux.svg[link="http://inkscape.org/doc/examples/tux.svg", opts=nofollow]

    // .image-with-loading-lazy
    image:sunset.jpg[loading=lazy]

    // .icon-font
    :icons: font
    icon:heart[]

    // .icon-font-with-title
    :icons: font
    icon:heart[title="I <3 Asciidoctor"]

    // .icon-font-with-size
    :icons: font
    icon:shield[2x]

    // .icon-font-with-rotate
    :icons: font
    icon:shield[rotate=90]

    // .icon-font-with-flip
    :icons: font
    icon:shield[flip=vertical]
  "#},
  html! {r##"
    <p>
      <a class="image" href="http://inkscape.org/doc/examples/tux.svg" target="_blank" rel="noopener">
        <img src="linux.svg" alt="linux">
      </a>
    </p>
    <p>
      <a class="image" href="http://inkscape.org/doc/examples/tux.svg">
        <img src="linux.svg" alt="linux">
      </a>
    </p>
    <p>
      <a class="image" href="http://inkscape.org/doc/examples/tux.svg" rel="nofollow">
        <img src="linux.svg" alt="linux">
      </a>
    </p>
    <p><img src="sunset.jpg" alt="sunset" loading="lazy"></p>
    <p><i class="fa fa-heart"></i></p>
    <p><i class="fa fa-heart" title="I &lt;3 Asciidoctor"></i></p>
    <p><i class="fa fa-shield fa-2x"></i></p>
    <p><i class="fa fa-shield fa-rotate-90"></i></p>
    <p><i class="fa fa-shield fa-flip-vertical"></i></p>
  "##}
);

assert_html!(
  roles,
  adoc! {r#"
    // .role-line-through
    [line-through]#striked text#

    // .role-strike
    [strike]#striked text#

    // .role-del
    [del]#deleted text#

    // .role-ins
    [ins]#inserted text#
  "#},
  html! {r##"
    <p><s>striked text</s></p>
    <p><s>striked text</s></p>
    <p><del>deleted text</del></p>
    <p><ins>inserted text</ins></p>
  "##}
);

assert_html!(
  quotes_cs,
  adoc! {r#"
      :lang: cs
      "`chunky bacon`"

      '`chunky bacon`'
    "#},
  html! {r##"
      <p>&#x201e;chunky bacon&#x201c;</p>
      <p>&#x201a;chunky bacon&#x2018;</p>
    "##}
);

assert_html!(
  quotes_fi,
  adoc! {r#"
    :lang: fi
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201d;chunky bacon&#x201d;</p>
    <p>&#x2019;chunky bacon&#x2019;</p>
  "##}
);

assert_html!(
  quotes_nl,
  adoc! {r#"
    :lang: nl
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201e;chunky bacon&#x201d;</p>
    <p>&#x201a;chunky bacon&#x2019;</p>
  "##}
);

assert_html!(
  quotes_pl,
  adoc! {r#"
    :lang: pl
    "`chunky bacon`"

    '`chunky bacon`'
  "#},
  html! {r##"
    <p>&#x201e;chunky bacon&#x201d;</p>
    <p>&#x00ab;chunky bacon&#x00bb;</p>
  "##}
);

assert_html!(
  source,
  adoc! {r#"
    [source, ruby]
    ----
    5.times do
      print "Odelay!"
    end
    ----

    [source, ruby, options="nowrap"]
    ----
    5.times do
      print "Odelay!"
    end
    ----
  "#},
  raw_html! {r##"
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">5.times do
      print "Odelay!"
    end</code></pre></div><div class="listing-block"><pre class="highlight nowrap"><code class="language-ruby" data-lang="ruby">5.times do
      print "Odelay!"
    end</code></pre></div>"##}
);

assert_html!(
  outline,
  adoc! {r#"
    // .sections-with-ids
    = Document Title
    :toc:

    == [[un]]Section _One_

    content one

    == [[two]][[deux]]Section Two

    content two

    == https://www.cvut.cz[*CTU* in Prague]

    content three
  "#},
  html! {r##"
    <nav id="toc" class="toc" role="doc-toc">
      <h2 id="toc-title">Table of Contents</h2>
      <ol class="toc-list level-1">
        <li><a href="#_section_one">Section <em>One</em></a></li>
        <li><a href="#_section_two">Section Two</a></li>
        <li><a href="#_httpswww_cvut_czctu_in_prague"><strong>CTU</strong> in Prague</a></li>
      </ol>
    </nav>
    <section class="doc-section level-1">
      <h2 id="_section_one"><a id="un" aria-hidden="true"></a>Section <em>One</em></h2>
      <p>content one</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="_section_two">
        <a id="two" aria-hidden="true"></a>
        <a id="deux" aria-hidden="true"></a>Section Two
      </h2>
      <p>content two</p>
    </section>
    <section class="doc-section level-1">
      <h2 id="_httpswww_cvut_czctu_in_prague">
        <a href="https://www.cvut.cz"><strong>CTU</strong> in Prague</a>
      </h2>
      <p>content three</p>
    </section>
  "##}
);

assert_html!(
  regressions,
  adoc! {r#"
    // .issue-10-two-sources-with-collist
    [source]
    ----
    source 1 line 1 // <1>
    source 1 line 2 // <2>
    ----
    <1> source 1 callout 1
    <2> source 1 callout 2

    Some text in here.
    
    [source]
    ----
    source 2 line 1 // <1>
    source 2 line 2 // <2>
    ----
    <1> source 2 callout 1
    <2> source 2 callout 2
    
    Where is this?

    == Heading
    
    This is actually here.
    
    // .issue-14-duplicated-footnotes-in-table
    |===
    
    |cell footnote:intable[first]
    
    |cell footnote:intable[]
    |===
    
    paragraph footnote:notintable[second]
    
    another paragraph footnote:notintable[]
  "#},
  raw_html! {r##"
    <div class="listing-block"><pre class="highlight"><code>source 1 line 1 // <b class="conum">1</b>
    source 1 line 2 // <b class="conum">2</b></code></pre><ol class="callout-list arabic"><li>source 1 callout 1</li><li>source 1 callout 2</li></ol></div><p>Some text in here.</p><div class="listing-block"><pre class="highlight"><code>source 2 line 1 // <b class="conum">1</b>
    source 2 line 2 // <b class="conum">2</b></code></pre><ol class="callout-list arabic"><li>source 2 callout 1</li><li>source 2 callout 2</li></ol></div><p>Where is this?</p><section class="doc-section level-1"><h2 id="_heading">Heading</h2><p>This is actually here.</p><div class="table-block"><table class="frame-all grid-all stretch"><colgroup><col style="width: 100%;"></colgroup><tbody><tr><td class="halign-left valign-top">cell <a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></td></tr><tr><td class="halign-left valign-top">cell <a class="footnote-ref" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a></td></tr></tbody></table></div><p>paragraph <a class="footnote-ref" id="_footnoteref_2" href="#_footnote_2" title="View footnote 2" role="doc-noteref">[2]</a></p><p>another paragraph <a class="footnote-ref" href="#_footnote_2" title="View footnote 2" role="doc-noteref">[2]</a></p></section><section class="footnotes" aria-label="Footnotes" role="doc-endnotes"><hr><ol class="footnotes"><li class="footnote" id="_footnote_1" role="doc-endnote">first <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li><li class="footnote" id="_footnote_2" role="doc-endnote">second <a class="footnote-backref" href="#_footnoteref_2" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a></li></ol></section>"##}
);

assert_html!(
  replacements,
  adoc! {r#"
    // .en-dash
    chunky -- bacon

    // .en-dash-at-bol
    -- bacon

    // .en-dash-at-eol
    chunky --

    // .en-dash-no-spaces
    1955--2011

    // .em-dash
    chunky --- bacon

    // .em-dash-at-bol
    --- bacon

    // .em-dash-at-eol
    chunky ---

    // .em-dash-no-spaces
    1955---2011
  "#},
  html! {r##"
    <p>chunky&#8201;&#8211;&#8201;bacon</p>
    <p>&#8201;&#8211;&#8201;bacon</p>
    <p>chunky&#8201;&#8211;&#8201;</p>
    <p>1955&#8211;2011</p>
    <p>chunky&#8201;&#8212;&#8201;bacon</p>
    <p>&#8201;&#8212;&#8201;bacon</p>
    <p>chunky&#8201;&#8212;&#8201;</p>
    <p>1955&#8212;&#8203;2011</p>
  "##}
);

assert_html!(
  book_gnarly_toc,
  adoc! {r#"
    = Book Title
    :doctype: book
    :sectnums:
    :toc:

    = First Part

    == Chapter

    === Subsection

    == Second Part

    == Chapter

    [appendix]
    = First Appendix

    === First Subsection

    === Second Subsection

    [appendix]
    = Second Appendix
  "#},
  html! {r##"
    <nav id="toc" class="toc" role="doc-toc">
      <h2 id="toc-title">Table of Contents</h2>
      <ol class="toc-list level-0">
        <li>
          <a href="#_first_part">First Part</a>
          <ol class="toc-list level-1">
            <li>
              <a href="#_chapter">1. Chapter</a>
              <ol class="toc-list level-2">
                <li><a href="#_subsection">1.1. Subsection</a></li>
              </ol>
            </li>
            <li><a href="#_second_part">2. Second Part</a></li>
            <li><a href="#_chapter_2">3. Chapter</a></li>
          </ol>
        </li>
        <li>
          <a href="#_first_appendix">Appendix A: First Appendix</a>
          <ol class="toc-list level-2">
            <li><a href="#_first_subsection">A.1. First Subsection</a></li>
            <li><a href="#_second_subsection">A.2. Second Subsection</a></li>
          </ol>
        </li>
        <li><a href="#_second_appendix">Appendix B: Second Appendix</a></li>
      </ol>
    </nav>
    <section class="doc-section level-0">
      <h1 id="_first_part">First Part</h1>
      <section class="doc-section level-1">
        <h2 id="_chapter">1. Chapter</h2>
        <section class="doc-section level-2">
          <h3 id="_subsection">1.1. Subsection</h3>
        </section>
      </section>
      <section class="doc-section level-1">
        <h2 id="_second_part">2. Second Part</h2>
      </section>
      <section class="doc-section level-1">
        <h2 id="_chapter_2">3. Chapter</h2>
      </section>
    </section>
    <section class="doc-section level-1">
      <h2 id="_first_appendix">Appendix A: First Appendix</h2>
      <section class="doc-section level-2">
        <h3 id="_first_subsection">A.1. First Subsection</h3>
      </section>
      <section class="doc-section level-2">
        <h3 id="_second_subsection">A.2. Second Subsection</h3>
      </section>
    </section>
    <section class="doc-section level-1">
      <h2 id="_second_appendix">Appendix B: Second Appendix</h2>
    </section>
  "##}
);

assert_html!(
  basic_callouts,
  adoc! {r#"
    [source,ruby]
    ----
    require 'sinatra' <1>

    get '/hi' do <2> <3>
      "Hello World!"
    end
    ----
  "#},
  raw_html! {r#"
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra' <b class="conum">1</b>

    get '/hi' do <b class="conum">2</b> <b class="conum">3</b>
      "Hello World!"
    end</code></pre></div>"#}
);

assert_html!(
  delimited_quote,
  adoc! {r#"
    [quote,Monty Python and the Holy Grail]
    ____
    Dennis: Come and see the violence inherent in the system. Help! Help!

    King Arthur: Bloody peasant!
    ____
  "#},
  html! {r#"
    <div class="quote-block">
      <blockquote>
        <p>Dennis: Come and see the violence inherent in the system. Help! Help!</p>
        <p>King Arthur: Bloody peasant!</p>
        <footer>&#8212; <cite>Monty Python and the Holy Grail</cite></footer>
      </blockquote>
    </div>
  "#}
);

assert_html!(
  simple_nested_desc_list,
  adoc! {r#"
    term1:: def1
    label1::: detail1
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt>term1</dt>
        <dd>def1
          <dl>
            <dt>label1</dt>
            <dd>detail1</dd>
          </dl>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  footnotes,
  adoc! {r#"
    foo.footnote:[bar _baz_]

    lol.footnote:cust[baz]
  "#},
  html! {r##"
    <p>
      foo.<a class="footnote-ref" id="_footnoteref_1" href="#_footnote_1" title="View footnote 1" role="doc-noteref">[1]</a>
    </p>
    <p>
      lol.<a class="footnote-ref" id="_footnoteref_2" href="#_footnote_2" title="View footnote 2" role="doc-noteref">[2]</a>
    </p>
    <section class="footnotes" aria-label="Footnotes" role="doc-endnotes">
      <hr>
      <ol class="footnotes">
        <li class="footnote" id="_footnote_1" role="doc-endnote">
          bar <em>baz</em> <a class="footnote-backref" href="#_footnoteref_1" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a>
        </li>
        <li class="footnote" id="_footnote_2" role="doc-endnote">
          baz <a class="footnote-backref" href="#_footnoteref_2" role="doc-backlink" title="Jump to the first occurrence in the text">&#8617;</a>
        </li>
      </ol>
    </section>
  "##}
);

assert_html!(
  image_links,
  adoc! {r#"
    [link=https://example.org]
    image::logo.png[Logo]

    image::logo.png[Logo,link=https://example.org]

    image:apply.jpg[Apply,link=https://apply.example.org] today!
  "#},
  html! {r#"
    <div class="image-block">
      <a class="image" href="https://example.org"><img src="logo.png" alt="Logo"></a>
    </div>
    <div class="image-block">
      <a class="image" href="https://example.org"><img src="logo.png" alt="Logo"></a>
    </div>
    <p>
      <a class="image" href="https://apply.example.org"><img src="apply.jpg" alt="Apply"></a> today!
    </p>
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
  abstract_block_style,
  adoc! {r#"
    = Document Title

    [abstract]
    .Abstract
    Pithy quote

    == First Section
  "#},
  html! {r#"
    <section id="preamble" aria-label="Preamble">
      <section class="quote-block abstract">
        <h6 class="block-title">Abstract</h6>
        <blockquote>Pithy quote</blockquote>
      </section>
    </section>
    <section class="doc-section level-1">
      <h2 id="_first_section">First Section</h2>
    </section>
  "#}
);

assert_standalone_body!(
  revision_marks,
  adoc! {r#"
    = The Intrepid Chronicles
    Kismet Lee
    2.9, October 31, 2021: Fall incarnation
  "#},
  html! {r#"
    <body class="article">
      <header>
        <h1>The Intrepid Chronicles</h1>
        <div class="details">
          <span class="author" id="author">Kismet Lee</span><br>
          <span id="revnumber">version 2.9,</span> <time id="revdate" datetime="2021-10-31">October 31, 2021</time><br>
          <span id="revremark">Fall incarnation</span>
        </div>
      </header>
      <div id="content"></div>
      <footer><div id="footer-text">Version 2.9</div></footer>
    </body>
  "#}
);

assert_standalone_body!(
  doc_authors,
  adoc! {r#"
    = Document Title
    Bob Smith <bob@smith.com>; Kate Smith; Henry Sue <henry@sue.com>
  "#},
  html! {r#"
    <body class="article">
      <header>
        <h1>Document Title</h1>
        <div class="details">
          <span class="author" id="author">Bob Smith</span><br>
          <span class="email" id="email"><a href="mailto:bob@smith.com">bob@smith.com</a></span><br>
          <span class="author" id="author2">Kate Smith</span><br>
          <span class="author" id="author3">Henry Sue</span><br>
          <span class="email" id="email3"><a href="mailto:henry@sue.com">henry@sue.com</a></span>
        </div>
      </header>
      <div id="content"></div>
      <footer><div id="footer-text"></div></footer>
    </body>
  "#}
);
