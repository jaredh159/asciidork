use test_utils::{adoc, html};

assert_html!(
  most_basic_unordered_list,
  adoc! {r#"
    * foo
    * bar
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li><p>foo</p></li>
        <li><p>bar</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  comment_separated_lists,
  adoc! {r#"
    * Apples
    * Oranges

    //-

    * Walnuts
    * Almonds
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li><p>Apples</p></li>
        <li><p>Oranges</p></li>
      </ul>
    </div>
    <div class="ulist">
      <ul>
        <li><p>Walnuts</p></li>
        <li><p>Almonds</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  multiline_list_principle_w_indent,
  adoc! {r#"
    * foo _bar_
      so *baz*
    * two
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li><p>foo <em>bar</em> so <strong>baz</strong></p></li>
        <li><p>two</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  simple_nested_list,
  adoc! {r#"
    * foo
    ** bar
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>foo</p>
          <div class="ulist">
            <ul>
              <li><p>bar</p></li>
            </ul>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  dashed_list,
  adoc! {r#"
    - Edgar Allan Poe
    - Sheri S. Tepper
    - Bill Bryson
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li><p>Edgar Allan Poe</p></li>
        <li><p>Sheri S. Tepper</p></li>
        <li><p>Bill Bryson</p></li>
      </ul>
    </div>
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
    <div class="ulist">
      <div class="title">Kizmets Favorite Authors</div>
      <ul>
        <li><p>Edgar Allan Poe</p></li>
        <li><p>Sheri S. Tepper</p></li>
        <li><p>Bill Bryson</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_custom_marker,
  adoc! {r#"
    [square]
    * Level 1 list item
    - Level 2 list item
    * Level 1 list item
  "#},
  html! {r#"
    <div class="ulist square">
      <ul class="square">
        <li>
          <p>Level 1 list item</p>
          <div class="ulist">
            <ul>
              <li><p>Level 2 list item</p></li>
            </ul>
          </div>
        </li>
        <li><p>Level 1 list item</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  nested_list_example,
  adoc! {r#"
    .Possible DefOps manual locations
    * West wood maze
    ** Maze heart
    *** Reflection pool
    ** Secret exit
    * Untracked file in git repository
  "#},
  html! {r#"
    <div class="ulist">
      <div class="title">Possible DefOps manual locations</div>
      <ul>
        <li>
          <p>West wood maze</p>
          <div class="ulist">
            <ul>
              <li>
                <p>Maze heart</p>
                <div class="ulist">
                  <ul>
                    <li><p>Reflection pool</p></li>
                  </ul>
                </div>
              </li>
              <li><p>Secret exit</p></li>
            </ul>
          </div>
        </li>
        <li><p>Untracked file in git repository</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_marker_mid_override,
  adoc! {r#"
    [square]
    * squares
    ** up top
    [circle]
    *** circles
    **** down below
  "#},
  html! {r#"
    <div class="ulist square">
      <ul class="square">
        <li>
          <p>squares</p>
          <div class="ulist">
            <ul>
              <li>
                <p>up top</p>
                <div class="ulist circle">
                  <ul class="circle">
                    <li>
                      <p>circles</p>
                      <div class="ulist">
                        <ul>
                          <li>
                            <p>down below</p>
                          </li>
                        </ul>
                      </div>
                    </li>
                  </ul>
                </div>
              </li>
            </ul>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_numbered,
  adoc! {r#"
    1. Protons
    2. Electrons
    3. Neutrons
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>Protons</p></li>
        <li><p>Electrons</p></li>
        <li><p>Neutrons</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_numbered_manual_start,
  adoc! {r#"
    4. Protons
    5. Electrons
    6. Neutrons
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic" start="4">
        <li><p>Protons</p></li>
        <li><p>Electrons</p></li>
        <li><p>Neutrons</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  numbered_list_attr_start,
  adoc! {r#"
    [start=4]
    . Protons
    . Electrons
    . Neutrons
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic" start="4">
        <li><p>Protons</p></li>
        <li><p>Electrons</p></li>
        <li><p>Neutrons</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  reversed_ordered_list,
  adoc! {r#"
    [%reversed]
    .Parts of an atom
    . Protons
    . Electrons
    . Neutrons
  "#},
  html! {r#"
    <div class="olist arabic">
      <div class="title">Parts of an atom</div>
      <ol class="arabic" reversed>
        <li><p>Protons</p></li>
        <li><p>Electrons</p></li>
        <li><p>Neutrons</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_nested_ordered,
  adoc! {r#"
    . Step 1
    . Step 2
    .. Step 2a
    .. Step 2b
    . Step 3
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>Step 1</p></li>
        <li>
          <p>Step 2</p>
          <div class="olist loweralpha">
            <ol class="loweralpha" type="a">
              <li><p>Step 2a</p></li>
              <li><p>Step 2b</p></li>
            </ol>
          </div>
        </li>
        <li><p>Step 3</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_unordered_within_ordered,
  adoc! {r#"
    . Linux
    * Fedora
    * Ubuntu
    * Slackware
    . BSD
    * FreeBSD
    * NetBSD
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li>
          <p>Linux</p>
          <div class="ulist">
            <ul>
              <li><p>Fedora</p></li>
              <li><p>Ubuntu</p></li>
              <li><p>Slackware</p></li>
            </ul>
          </div>
        </li>
        <li>
          <p>BSD</p>
          <div class="ulist">
            <ul>
              <li><p>FreeBSD</p></li>
              <li><p>NetBSD</p></li>
            </ul>
          </div>
        </li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_unordered_within_ordered_spaced,
  adoc! {r#"
    . Linux

      * Fedora
      * Ubuntu
      * Slackware

    . BSD

      * FreeBSD
      * NetBSD
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li>
          <p>Linux</p>
          <div class="ulist">
            <ul>
              <li><p>Fedora</p></li>
              <li><p>Ubuntu</p></li>
              <li><p>Slackware</p></li>
            </ul>
          </div>
        </li>
        <li>
          <p>BSD</p>
          <div class="ulist">
            <ul>
              <li><p>FreeBSD</p></li>
              <li><p>NetBSD</p></li>
            </ul>
          </div>
        </li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_ordered_marker_override,
  adoc! {r#"
    [lowerroman,start=5]
    . Five
    . Six
    [loweralpha]
    .. a
    .. b
    .. c
    . Seven
  "#},
  html! {r#"
    <div class="olist lowerroman">
      <ol class="lowerroman" type="i" start="5">
        <li><p>Five</p></li>
        <li>
          <p>Six</p>
          <div class="olist loweralpha">
            <ol class="loweralpha" type="a">
              <li><p>a</p></li>
              <li><p>b</p></li>
              <li><p>c</p></li>
            </ol>
          </div>
        </li>
        <li><p>Seven</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  checklist,
  adoc! {r#"
    [.custom-class]
    * [*] checked
    * [x] also checked
    * [ ] not checked
    * normal list item
  "#},
  html! {r#"
    <div class="ulist checklist custom-class">
      <ul class="checklist">
        <li><p>&#10003; checked</p></li>
        <li><p>&#10003; also checked</p></li>
        <li><p>&#10063; not checked</p></li>
        <li><p>normal list item</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_interactive_checklist,
  adoc! {r#"
    [%interactive]
    * [*] checked
    * [x] also checked
    * [ ] not checked
  "#},
  html! {r#"
    <div class="ulist checklist">
      <ul class="checklist">
        <li><p><input type="checkbox" data-item-complete="1" checked> checked</p></li>
        <li><p><input type="checkbox" data-item-complete="1" checked> also checked</p></li>
        <li><p><input type="checkbox" data-item-complete="0"> not checked</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  ordered_list_not_checklist,
  adoc! {r#"
    . [*] checked
    . [ ] not checked
  "#},
  html! {r#"
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>[*] checked</p></li>
        <li><p>[ ] not checked</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_item_continuation,
  adoc! {r#"
    * principle
    +
    with continuation
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>principle</p>
          <div class="paragraph">
            <p>with continuation</p>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_item_2_continuations,
  adoc! {r#"
    * principle
    +
    with continuation
    +
    and another
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>principle</p>
          <div class="paragraph">
            <p>with continuation</p>
          </div>
          <div class="paragraph">
            <p>and another</p>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_nested_inside_continuation,
  adoc! {r#"
    * Item 1
    +
    Foo

    ** Item 1.1
    ** Item 1.2

    * Item 2
    +
    Baz
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>Item 1</p>
          <div class="paragraph"><p>Foo</p></div>
          <div class="ulist">
            <ul>
              <li><p>Item 1.1</p></li>
              <li><p>Item 1.2</p></li>
            </ul>
          </div>
        </li>
        <li>
          <p>Item 2</p>
          <div class="paragraph"><p>Baz</p></div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_items_w_delimited_listing_blocks,
  adoc! {r#"
    * item 1
    +
    ----
    cont 1
    ----

    * item 2
    +
    ----
    cont 2
    ----
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>item 1</p>
          <div class="listingblock">
            <div class="content">
              <pre>cont 1</pre>
            </div>
          </div>
        </li>
        <li>
          <p>item 2</p>
          <div class="listingblock">
            <div class="content">
              <pre>cont 2</pre>
            </div>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_items_w_delimited_blocks,
  adoc! {r#"
    * principle
    +
    --
    para 1

    para 2
    --

    * another item
    +
    --
    para 3

    para 4
    --
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li>
          <p>principle</p>
          <div class="openblock">
            <div class="content">
              <div class="paragraph"><p>para 1</p></div>
              <div class="paragraph"><p>para 2</p></div>
            </div>
          </div>
        </li>
        <li>
          <p>another item</p>
          <div class="openblock">
            <div class="content">
              <div class="paragraph"><p>para 3</p></div>
              <div class="paragraph"><p>para 4</p></div>
            </div>
          </div>
        </li>
      </ul>
    </div>
  "#}
);

assert_html!(
  list_empty_principle,
  adoc! {r#"
     . {empty}
     +
     --
     para
     --
   "#},
  html! {r#"
     <div class="olist arabic">
       <ol class="arabic">
         <li>
           <p></p>
           <div class="openblock">
             <div class="content">
               <div class="paragraph"><p>para</p></div>
             </div>
           </div>
         </li>
       </ol>
     </div>
   "#}
);

assert_html!(
  list_item_principle_from_attr_ref,
  adoc! {r#"
    para

    :foo: bar

    . {foo}
  "#},
  html! {r#"
    <div class="paragraph"><p>para</p></div>
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>bar</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  list_item_principle_from_multiline_attr_ref,
  adoc! {r#"
    para

    :foo: one \
    and two \
    and three

    . {foo}
  "#},
  html! {r#"
    <div class="paragraph"><p>para</p></div>
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>one and two and three</p></li>
      </ol>
    </div>
  "#}
);

assert_html!(
  complex_continuation_example,
  adoc! {r#"
    * The header in AsciiDoc must start with a document title.
    +
    ----
    = Document Title
    ----
    +
    Keep in mind that the header is optional.

    * Optional author and revision information lines immediately follow the document title.
    +
    ----
    = Document Title
    Doc Writer <doc.writer@asciidoc.org>
    v1.0, 2022-01-01
    ----
  "#},
  r#"<div class="ulist"><ul><li><p>The header in AsciiDoc must start with a document title.</p><div class="listingblock"><div class="content"><pre>= Document Title</pre></div></div><div class="paragraph"><p>Keep in mind that the header is optional.</p></div></li><li><p>Optional author and revision information lines immediately follow the document title.</p><div class="listingblock"><div class="content"><pre>= Document Title
Doc Writer &lt;doc.writer@asciidoc.org&gt;
v1.0, 2022-01-01</pre></div></div></li></ul></div>"#
);

assert_html!(
  list_items_separated_by_comment_block,
  adoc! {r#"
    * first item
    +
    ////
    A comment block in a list.

    Notice it's attached to the preceding list item.
    ////

    * second item
  "#},
  html! {r#"
    <div class="ulist">
      <ul>
        <li><p>first item</p></li>
        <li><p>second item</p></li>
      </ul>
    </div>
  "#}
);

assert_html!(
  bibliography_section_list,
  adoc! {r#"
    == Title

    A <<foo>> and a <<b123>>.

    Out of context [[[bib-ref]]] not recognized.

    [bibliography]
    == References

    * [[[foo]]] foo
    * [[[b123, 1]]] bar

    // break

    * no bib anchors
    * but should have bibliography class
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="_title">Title</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>A <a href="#foo">[foo]</a> and a <a href="#b123">[1]</a>.</p>
        </div>
        <div class="paragraph">
          <p>Out of context [<a id="bib-ref"></a>] not recognized.</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_references">References</h2>
      <div class="sectionbody">
        <div class="ulist bibliography">
          <ul class="bibliography">
            <li><p><a id="foo"></a>[foo] foo</p></li>
            <li><p><a id="b123"></a>[1] bar</p></li>
          </ul>
        </div>
        <div class="ulist bibliography">
          <ul class="bibliography">
            <li><p>no bib anchors</p></li>
            <li><p>but should have bibliography class</p></li>
          </ul>
        </div>
      </div>
    </div>
  "##}
);

assert_html!(
  // not documented, but dan said this is valid in zulip
  bibliography_list_not_in_section,
  adoc! {r#"
    [bibliography]
    - [[[taoup]]] Eric Steven Raymond. _The Art of Unix
      Programming_. Addison-Wesley. ISBN 0-13-142901-9.
    - [[[walsh-muellner]]] Norman Walsh & Leonard Muellner.
      _DocBook - The Definitive Guide_. O'Reilly & Associates. 1999.
      ISBN 1-56592-580-7.
  "#},
  html! {r#"
    <div class="ulist bibliography">
      <ul class="bibliography">
        <li>
          <p><a id="taoup"></a>[taoup] Eric Steven Raymond. <em>The Art of Unix Programming</em>. Addison-Wesley. ISBN 0-13-142901-9.</p>
        </li>
        <li>
          <p><a id="walsh-muellner"></a>[walsh-muellner] Norman Walsh &amp; Leonard Muellner. <em>DocBook - The Definitive Guide</em>. O&#8217;Reilly &amp; Associates. 1999. ISBN 1-56592-580-7.</p>
        </li>
        </ul>
    </div>
  "#}
);
