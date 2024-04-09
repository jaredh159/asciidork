use test_utils::{adoc, html};
mod helpers;

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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

test_eval!(
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
