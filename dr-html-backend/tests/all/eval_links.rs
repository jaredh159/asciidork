use test_utils::*;

assert_html!(
  link_macros,
  adoc! {r#"
    Visit https://site.com for more.

    Or click link:report.pdf[here _son_].

    Brackets: <http://example.com> too.

    Escaped is not link: \http://nolink.com

    Email me at me@example.com as well.

    link:https://example.org/dist/info.adoc[role=include]

    [subs=-macros]
    Not processed: https://site.com

    https://chat.asciidoc.org[Discuss AsciiDoc,role=resource,window=_blank]

    https://example.com[window=_blank,opts=nofollow]

    link:post.html[My Post,opts=nofollow]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Visit <a href="https://site.com" class="bare">https://site.com</a> for more.</p>
    </div>
    <div class="paragraph">
      <p>Or click <a href="report.pdf">here <em>son</em></a>.</p>
    </div>
    <div class="paragraph">
      <p>Brackets: <a href="http://example.com" class="bare">http://example.com</a> too.</p>
    </div>
    <div class="paragraph">
      <p>Escaped is not link: http://nolink.com</p>
    </div>
    <div class="paragraph">
      <p>Email me at <a href="mailto:me@example.com">me@example.com</a> as well.</p>
    </div>
    <div class="paragraph">
      <p>
        <a href="https://example.org/dist/info.adoc" class="bare include">https://example.org/dist/info.adoc</a>
      </p>
    </div>
    <div class="paragraph">
      <p>Not processed: https://site.com</p>
    </div>
    <div class="paragraph">
      <p><a href="https://chat.asciidoc.org" class="resource" target="_blank" rel="noopener">Discuss AsciiDoc</a></p>
    </div>
    <div class="paragraph">
      <p><a href="https://example.com" class="bare" target="_blank" rel="noopener nofollow">https://example.com</a></p>
    </div>
    <div class="paragraph">
      <p><a href="post.html" rel="nofollow">My Post</a></p>
    </div>
  "#}
);

assert_html!(
  blank_window_shorthand,
  adoc! {r#"
    View html: link:view-source:asciidoctor.org[Asciidoctor homepage^].

    https://example.org["Google, DuckDuckGo, Ecosia^",role=btn]

    https://example.org[Google, DuckDuckGo, Ecosia^]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>View html: <a href="view-source:asciidoctor.org" target="_blank" rel="noopener">Asciidoctor homepage</a>.</p>
    </div>
    <div class="paragraph">
      <p><a href="https://example.org" class="btn" target="_blank" rel="noopener">Google, DuckDuckGo, Ecosia</a></p>
    </div>
    <div class="paragraph">
      <p><a href="https://example.org" target="_blank" rel="noopener">Google, DuckDuckGo, Ecosia</a></p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb1,
  adoc! {r#"
    // qualified url inline with text
    The AsciiDoc project is located at http://asciidoc.org.

    // qualified url with role inline with text
    The AsciiDoc project is located at http://asciidoc.org[role=project].

    // qualified http url inline with hide-uri-scheme set
    :hide-uri-scheme:
    The AsciiDoc project is located at http://asciidoc.org.

    // qualified file url inline with hide-uri-scheme set
    Edit the configuration file link:file:///etc/app.conf[]

    // should not hide bare uri scheme in implicit text when hide-uri-scheme set
    :!hide-uri-scheme:
    foo link:https://[] bar link:ssh://[]

    // qualified file url inline with label
    file:///home/user/bookmarks.html[My Bookmarks]

    // qualified url with label
    We're parsing http://asciidoc.org[AsciiDoc] markup

    // qualified url with label containing escaped right square bracket
    We're parsing http://asciidoc.org[[Ascii\]Doc] markup

    // qualified url with backslash label
    I advise you to https://google.com[Google for +\+]

    // qualified url with label using link macro
    We're parsing link:http://asciidoc.org[AsciiDoc] markup

    // qualified url with role using link macro
    We're parsing link:http://asciidoc.org[role=project] markup

    // qualified url with label containing square brackets using link macro
    http://example.com[[bracket1\]]

    // link macro with empty target
    Link to link:[this page].

    // should not recognize link macro with double colons
    The link::http://example.org[example domain] is blah blah.

    // qualified url surrounded by angle brackets
    <http://asciidoc.org> is the project page for AsciiDoc.
  "#},
  html! {r#"
    <div class="paragraph">
      <p>The AsciiDoc project is located at <a href="http://asciidoc.org" class="bare">http://asciidoc.org</a>.</p>
    </div>
    <div class="paragraph">
      <p>The AsciiDoc project is located at <a href="http://asciidoc.org" class="bare project">http://asciidoc.org</a>.</p>
    </div>
    <div class="paragraph">
      <p>The AsciiDoc project is located at <a href="http://asciidoc.org" class="bare">asciidoc.org</a>.</p>
    </div>
    <div class="paragraph">
      <p>Edit the configuration file <a href="file:///etc/app.conf" class="bare">/etc/app.conf</a></p>
    </div>
    <div class="paragraph">
      <p>foo <a href="https://" class="bare">https://</a> bar <a href="ssh://" class="bare">ssh://</a></p>
    </div>
    <div class="paragraph">
      <p><a href="file:///home/user/bookmarks.html">My Bookmarks</a></p>
    </div>
    <div class="paragraph">
      <p>We&#8217;re parsing <a href="http://asciidoc.org">AsciiDoc</a> markup</p>
    </div>
    <div class="paragraph">
      <p>We&#8217;re parsing <a href="http://asciidoc.org">[Ascii]Doc</a> markup</p>
    </div>
    <div class="paragraph">
      <p>I advise you to <a href="https://google.com">Google for \</a></p>
    </div>
    <div class="paragraph">
      <p>We&#8217;re parsing <a href="http://asciidoc.org">AsciiDoc</a> markup</p>
    </div>
    <div class="paragraph">
      <p>We&#8217;re parsing <a href="http://asciidoc.org" class="bare project">http://asciidoc.org</a> markup</p>
    </div>
    <div class="paragraph">
      <p><a href="http://example.com">[bracket1]</a></p>
    </div>
    <div class="paragraph">
      <p>Link to <a href="">this page</a>.</p>
    </div>
    <div class="paragraph">
      <p>The link::<a href="http://example.org">example domain</a> is blah blah.</p>
    </div>
    <div class="paragraph">
      <p><a href="http://asciidoc.org" class="bare">http://asciidoc.org</a> is the project page for AsciiDoc.</p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb2,
  adoc! {r#"
    // qualified url surrounded by parens
    (http://foo.com) is bar.

    // qualified url with trailing period
    The homepage for Asciidoctor is https://asciidoctor.org.

    // qualified url with trailing explanation point
    Check out https://asciidoctor.org!

    // qualified url with trailing question mark
    Is the homepage for Asciidoctor https://asciidoctor.org?

    // qualified url with trailing round bracket
    Asciidoctor is a Ruby-based AsciiDoc processor (see https://asciidoctor.org)

    // qualified url with trailing period followed by round bracket
    (The homepage for Asciidoctor is https://asciidoctor.org.)

    // qualified url with trailing exclamation point followed by round bracket
    (Check out https://asciidoctor.org!)

    // qualified url with trailing question mark followed by round bracket
    (Is the homepage for Asciidoctor https://asciidoctor.org?)

    // qualified url with trailing semi-colon
    https://asciidoctor.org; where text gets parsed

    // qualified url with trailing colon
    https://asciidoctor.org: where text gets parsed

    // qualified url in round brackets with trailing colon
    (https://asciidoctor.org): where text gets parsed

    // qualified url with trailing round bracket followed by colon
    (from https://asciidoctor.org): where text gets parsed

    // qualified url in round brackets with trailing semi-colon
    (https://asciidoctor.org); where text gets parsed

    // qualified url with trailing round bracket followed by semi-colon
    (from https://asciidoctor.org); where text gets parsed
  "#},
  html! {r#"
    <div class="paragraph">
      <p>(<a href="http://foo.com" class="bare">http://foo.com</a>) is bar.</p>
    </div>
    <div class="paragraph">
      <p>The homepage for Asciidoctor is <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>.</p>
    </div>
    <div class="paragraph">
      <p>Check out <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>!</p>
    </div>
    <div class="paragraph">
      <p>Is the homepage for Asciidoctor <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>?</p>
    </div>
    <div class="paragraph">
      <p>Asciidoctor is a Ruby-based AsciiDoc processor (see <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>)</p>
    </div>
    <div class="paragraph">
      <p>(The homepage for Asciidoctor is <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>.)</p>
    </div>
    <div class="paragraph">
      <p>(Check out <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>!)</p>
    </div>
    <div class="paragraph">
      <p>(Is the homepage for Asciidoctor <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>?)</p>
    </div>
    <div class="paragraph">
      <p><a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>; where text gets parsed</p>
    </div>
    <div class="paragraph">
      <p><a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>: where text gets parsed</p>
    </div>
    <div class="paragraph">
      <p>(<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>): where text gets parsed</p>
    </div>
    <div class="paragraph">
      <p>(from <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>): where text gets parsed</p>
    </div>
    <div class="paragraph">
      <p>(<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>); where text gets parsed</p>
    </div>
    <div class="paragraph">
      <p>(from <a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>); where text gets parsed</p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb3,
  adoc! {r#"
    // these should not be converted
    (https://) http://; file://: <ftp://>

    // qualified url containing parens
    http://jruby.org/apidocs/org/jruby/Ruby.html#addModule(org.jruby.RubyModule)[addModule() adds a Ruby module]

    // qualified url adjacent to text in square brackets
    ]http://asciidoc.org[AsciiDoc] project page.

    // qualified url adjacent to text in round brackets
    )http://asciidoc.org[AsciiDoc] project page.
  "#},
  html! {r#"
    <div class="paragraph">
      <p>(https://) http://; file://: &lt;ftp://&gt;</p>
    </div>
    <div class="paragraph">
      <p><a href="http://jruby.org/apidocs/org/jruby/Ruby.html#addModule(org.jruby.RubyModule)">addModule() adds a Ruby module</a></p>
    </div>
    <div class="paragraph">
      <p>]<a href="http://asciidoc.org">AsciiDoc</a> project page.</p>
    </div>
    <div class="paragraph">
      <p>)<a href="http://asciidoc.org">AsciiDoc</a> project page.</p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_special_chars,
  "\u{00A0}http://asciidoc.org[AsciiDoc] project page.",
  contains: "\u{00A0}<a href=\"http://asciidoc.org\">AsciiDoc</a> project page.</p>"
);

// let string = r#"Hello\u{00A0}World"#;
// println!("{}", string); // Output: Hello World (with no-break space)
