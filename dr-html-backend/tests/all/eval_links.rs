use test_utils::*;

assert_html!(
  url_escaping,
  adoc! {r#"
    //                          v-----v -- these get encoded
    Use http://example.com?menu=<value>[] to open to the menu named `<value>`.
   
    //     vvvvv -- valid entity, no double encoding
    link:My&#32;Documents/report.pdf[Get Report]

    //        vvv -- not a valid entity, & -> &amp;
    link:Docum&#x;ents/report.pdf[Get Report]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>
        Use <a href="http://example.com?menu=&lt;value&gt;" class="bare">
          http://example.com?menu=&lt;value&gt;
        </a> to open to the menu named <code>&lt;value&gt;</code>.
      </p>
    </div>
    <div class="paragraph">
      <p><a href="My&#32;Documents/report.pdf">Get Report</a></p>
    </div>
    <div class="paragraph">
      <p><a href="Docum&amp;#x;ents/report.pdf">Get Report</a></p>
    </div>
  "#}
);

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
      <p><a href="https://chat.asciidoc.org" target="_blank" rel="noopener" class="resource">Discuss AsciiDoc</a></p>
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
      <p><a href="https://example.org" target="_blank" rel="noopener" class="btn">Google, DuckDuckGo, Ecosia</a></p>
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

    // qualified url following smart apostrophy
    l&#8217;http://www.irit.fr[IRIT]

    // qualified url macro enclosed in double quotes
    "https://asciidoctor.org[]"

    // qualified url macro enclosed in single quotes
    'https://asciidoctor.org[]'

    // qualified url macro with trailing period
    Information about the https://symbols.example.org/.[.] character.

    // escaped inline qualified url should not create link
    \http://escaped.com is not a link

    // escaped inline qualified url as macro should not create link
    \http://escaped.com[escaped.com] is not a link

    // url in link macro with at (@) sign should not create mailto link
    http://a.com/b/dev@foo.com[subscribe]

    // implicit url with at (@) sign should not create mailto link
    http://a.com/b/dev@foo.com
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
    <div class="paragraph">
      <p>l&#8217;<a href="http://www.irit.fr">IRIT</a></p>
    </div>
    <div class="paragraph">
      <p>"<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>"</p>
    </div>
    <div class="paragraph">
      <p>'<a href="https://asciidoctor.org" class="bare">https://asciidoctor.org</a>'</p>
    </div>
    <div class="paragraph">
      <p>Information about the <a href="https://symbols.example.org/.">.</a> character.</p>
    </div>
    <div class="paragraph">
      <p>http://escaped.com is not a link</p>
    </div>
    <div class="paragraph">
      <p>http://escaped.com[escaped.com] is not a link</p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.com/b/dev@foo.com">subscribe</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.com/b/dev@foo.com" class="bare">http://a.com/b/dev@foo.com</a></p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb4,
  adoc! {r#"
    // inline qualified url followed by a newline should not include newline in link
    Code is at http://github.com/foo
    which is a github organization.

    // qualified url using INVALID LINK MACRO should not create link
    link:http://foo.com

    // qualified url divided by NEWLINE using macro syntax should not create link
    Foo link:https://example.com
    [] is bar.

    // qualified url containing WHITESPACE using macro syntax SHOULD NOT create link
    Foo link:https://example.com?q=foo bar[] is bar.

    // qualified url containing an ENCODED SPACE using macro syntax SHOULD create a link
    Foo link:https://example.com?q=foo%20bar[] is bar.

    // inline quoted qualified url should not consume surrounding angled brackets
    Foo: <**https://foo.com/bar**>

    // link with quoted text should not be separated into attributes when text contains an equal sign
    http://foo.com["foo, bar = baz"]

    // link with comma in text but no equal sign should not be separated into attributes
    http://foo.com[foo, bar, baz]
  "#},
  html! {r#"
    <div class="paragraph">
      <p>Code is at <a href="http://github.com/foo" class="bare">http://github.com/foo</a> which is a github organization.</p>
    </div>
    <div class="paragraph">
      <p>link:http://foo.com</p>
    </div>
    <div class="paragraph">
      <p>Foo link:https://example.com [] is bar.</p>
    </div>
    <div class="paragraph">
      <p>Foo link:https://example.com?q=foo bar[] is bar.</p>
    </div>
    <div class="paragraph">
      <p>Foo <a href="https://example.com?q=foo%20bar" class="bare">https://example.com?q=foo%20bar</a> is bar.</p>
    </div>
    <div class="paragraph">
      <p>Foo: &lt;<strong><a href="https://foo.com/bar" class="bare">https://foo.com/bar</a></strong>&gt;</p>
    </div>
    <div class="paragraph">
      <p><a href="http://foo.com">foo, bar = baz</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://foo.com">foo, bar, baz</a></p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb5,
  adoc! {r#"
    // link with formatted wrapped text should not be separated into attributes
    https://example.com[[.role]#Foo Bar#]

    // should process role and window attributes on link
    http://google.com[Google, role=external, window="_blank"]

    // link macro with attributes but NO text should use URL as text
    link:http://a.com?b=c:1,2b,[family=c,weight=400]

    // link macro with attributes but BLANK text should use URL as text
    link:http://a.com?b=c:1,2b,[,family=c,weight=400]

    // link macro with comma but no explicit attributes in text should not parse text
    link:http://a.com?b=c:1,2b,[Roboto,400]

    // link macro should support id and role attributes
    link:http://example.com[,id=roboto-regular,role=font]

    // link text that ends in ^ should set link window to _blank
    http://google.com[Google^]

    // rel=noopener should be added to a link that targets a named window when the noopener option is set
    http://google.com[Google,window=name,opts=noopener]

    // rel=noopener should not be added to a link if it does not target a window
    http://google.com[Google,opts=noopener]

    // rel=nofollow should be added to a link when the nofollow option is set
    http://google.com[Google,window=name,opts="nofollow,noopener"]
  "#},
  html! {r#"
    <div class="paragraph">
      <p><a href="https://example.com"><span class="role">Foo Bar</span></a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com" target="_blank" rel="noopener" class="external">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.com?b=c:1,2b," class="bare">http://a.com?b=c:1,2b,</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.com?b=c:1,2b," class="bare">http://a.com?b=c:1,2b,</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://a.com?b=c:1,2b,">Roboto,400</a></p>
    </div>
    <div class="paragraph">
      <p><a id="roboto-regular" href="http://example.com" class="bare font">http://example.com</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com" target="_blank" rel="noopener">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com" target="name" rel="noopener">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com" target="name" rel="noopener nofollow">Google</a></p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_rb6,
  adoc! {r#"
    // id attribute on link is processed
    http://google.com[Google, id="link-1"]

    // title attribute on link is processed
    http://google.com[Google, title="title-1"]

    // inline irc link
    irc://irc.freenode.net

    // inline irc link with text
    irc://irc.freenode.net[Freenode IRC]
  "#},
  html! {r#"
    <div class="paragraph">
      <p><a id="link-1" href="http://google.com">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="http://google.com" title="title-1">Google</a></p>
    </div>
    <div class="paragraph">
      <p><a href="irc://irc.freenode.net" class="bare">irc://irc.freenode.net</a></p>
    </div>
    <div class="paragraph">
      <p><a href="irc://irc.freenode.net">Freenode IRC</a></p>
    </div>
  "#}
);

assert_html!(
  asciidoctor_links_test_special_chars,
  "\u{00A0}http://asciidoc.org[AsciiDoc] project page.",
  contains: "\u{00A0}<a href=\"http://asciidoc.org\">AsciiDoc</a> project page.</p>"
);

assert_html!(
  no_nested_link,
  "http://example.com/test1[http://example.com/test1]",
  html! {r#"
    <div class="paragraph">
      <p><a href="http://example.com/test1">http://example.com/test1</a></p>
    </div>
  "#}
);
