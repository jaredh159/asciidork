use asciidork_parser::prelude::*;
use test_utils::*;

assert_html!(
  simple_description_list_1,
  adoc! {r#"
    foo:: bar
    hash:: baz
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd><p>bar</p></dd>
        <dt class="hdlist1">hash</dt>
        <dd><p>baz</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  multiple_terms,
  adoc! {r#"
    foo::
    bar:: baz
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dt class="hdlist1">bar</dt>
        <dd><p>baz</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  glossary_dlist,
  adoc! {r#"
    [glossary]
    mud:: wet, cold dirt
  "#},
  html! {r#"
    <div class="dlist glossary">
      <dl>
        <dt>mud</dt>
        <dd><p>wet, cold dirt</p></dd>
      </dl>
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
        <dt class="hdlist1">term1</dt>
        <dd>
          <p>def1</p>
          <div class="dlist">
            <dl>
              <dt class="hdlist1">label1</dt>
              <dd><p>detail1</p></dd>
            </dl>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  no_term_text_but_simple_attached_block,
  adoc! {r#"
    term::
    +
    paragraph
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term</dt>
        <dd><div class="paragraph"><p>paragraph</p></div></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  desc_list_comment_not_confused_with_desc,
  adoc! {r#"
    category a::
    //ignored term:: def
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">category a</dt>
      </dl>
    </div>
  "#}
);

assert_html!(
  thematic_break_separates_desc_lists,
  adoc! {r#"
    foo:: bar

    '''

    baz:: qux
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd><p>bar</p></dd>
      </dl>
    </div>
    <hr>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">baz</dt>
        <dd><p>qux</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  simple_description_list_2,
  adoc! {r#"
    foo:: bar
    baz:: qux
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd><p>bar</p></dd>
        <dt class="hdlist1">baz</dt>
        <dd><p>qux</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  description_list_w_whitespace_para,
  adoc! {r#"
    foo::

    bar is
    so baz

    baz:: qux
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd><p>bar is so baz</p></dd>
        <dt class="hdlist1">baz</dt>
        <dd><p>qux</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  list_w_continuation,
  adoc! {r#"
    foo::
    bar so baz
    +
    and more things
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd>
          <p>bar so baz</p>
          <div class="paragraph">
            <p>and more things</p>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  list_w_double_continuation,
  adoc! {r#"
    foo::
    bar so baz
    +
    and more things
    +
    and even more things
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">foo</dt>
        <dd>
          <p>bar so baz</p>
          <div class="paragraph">
            <p>and more things</p>
          </div>
          <div class="paragraph">
            <p>and even more things</p>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  mixing_lists,
  adoc! {r#"
    Dairy::
    * Milk
    * Eggs
    Bakery::
    * Bread
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">Dairy</dt>
        <dd>
          <div class="ulist">
            <ul>
              <li><p>Milk</p></li>
              <li><p>Eggs</p></li>
            </ul>
          </div>
        </dd>
        <dt class="hdlist1">Bakery</dt>
        <dd>
          <div class="ulist">
            <ul>
              <li><p>Bread</p></li>
            </ul>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  mixing_lists_w_space,
  adoc! {r#"
    Dairy::

      * Milk
      * Eggs

    Bakery::

      * Bread
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">Dairy</dt>
        <dd>
          <div class="ulist">
            <ul>
              <li><p>Milk</p></li>
              <li><p>Eggs</p></li>
            </ul>
          </div>
        </dd>
        <dt class="hdlist1">Bakery</dt>
        <dd>
          <div class="ulist">
            <ul>
              <li><p>Bread</p></li>
            </ul>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  nested_description_list,
  adoc! {r#"
    Operating Systems::
      Linux:::
        . Fedora
          * Desktop
        . Ubuntu
          * Desktop
          * Server
      BSD:::
        . FreeBSD
        . NetBSD

    Cloud Providers::
      PaaS:::
        . OpenShift
        . CloudBees
      IaaS:::
        . Amazon EC2
        . Rackspace
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">Operating Systems</dt>
        <dd>
          <div class="dlist">
            <dl>
              <dt class="hdlist1">Linux</dt>
              <dd>
                <div class="olist arabic">
                  <ol class="arabic">
                    <li>
                      <p>Fedora</p>
                      <div class="ulist">
                        <ul>
                          <li><p>Desktop</p></li>
                        </ul>
                      </div>
                    </li>
                    <li>
                      <p>Ubuntu</p>
                      <div class="ulist">
                        <ul>
                          <li><p>Desktop</p></li>
                          <li><p>Server</p></li>
                        </ul>
                      </div>
                    </li>
                  </ol>
                </div>
              </dd>
              <dt class="hdlist1">BSD</dt>
              <dd>
                <div class="olist arabic">
                  <ol class="arabic">
                    <li><p>FreeBSD</p></li>
                    <li><p>NetBSD</p></li>
                  </ol>
                </div>
              </dd>
            </dl>
          </div>
        </dd>
        <dt class="hdlist1">Cloud Providers</dt>
        <dd>
          <div class="dlist">
            <dl>
              <dt class="hdlist1">PaaS</dt>
              <dd>
                <div class="olist arabic">
                  <ol class="arabic">
                    <li><p>OpenShift</p></li>
                    <li><p>CloudBees</p></li>
                  </ol>
                </div>
              </dd>
              <dt class="hdlist1">IaaS</dt>
              <dd>
                <div class="olist arabic">
                  <ol class="arabic">
                    <li><p>Amazon EC2</p></li>
                    <li><p>Rackspace</p></li>
                  </ol>
                </div>
              </dd>
            </dl>
          </div>
        </dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  literal_block_inside_desc_list,
  adoc! {r#"
    // literal block inside description list
    term::
    +
    ....
    literal, line 1

    literal, line 2
    ....
    anotherterm:: def
  "#},
  "<div class=\"dlist\"><dl><dt class=\"hdlist1\">term</dt><dd><div class=\"literalblock\"><div class=\"content\"><pre>literal, line 1\n\nliteral, line 2</pre></div></div></dd><dt class=\"hdlist1\">anotherterm</dt><dd><p>def</p></dd></dl></div>"
);

assert_html!(
  trailing_continuation_desc,
  strict: false,
  adoc! {r#"
    // literal block inside description list with trailing line continuation
    term::
    +
    ....
    literal
    ....
    +
    anotherterm:: def
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term</dt>
        <dd>
          <div class="literalblock">
            <div class="content"><pre>literal</pre></div>
          </div>
        </dd>
        <dt class="hdlist1">anotherterm</dt>
        <dd><p>def</p></dd>
      </dl>
    </div>
  "#}
);

assert_error!(
  trailing_continuation_desc_err,
  adoc! {r#"
    term::
    +
    ....
    literal
    ....
    +
    anotherterm:: def
  "#},
  error! {r"
     --> test.adoc:6:1
      |
    6 | +
      | ^ Dangling list continuation
  "}
);

// multiple listing blocks inside description list
assert_html!(
  multiple_listing_continuations,
  adoc! {r#"
    term::
    +
    ----
    listing1
    ----
    +
    ----
    listing2
    ----
    anotherterm:: def
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term</dt>
        <dd>
          <div class="listingblock">
            <div class="content"><pre>listing1</pre></div>
          </div>
          <div class="listingblock">
            <div class="content"><pre>listing2</pre></div>
          </div>
        </dd>
        <dt class="hdlist1">anotherterm</dt>
        <dd><p>def</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  adjacent_tabbed,
  // single-line indented adjacent elements with tabs
  "term1::\tdef1\n\tterm2::\tdef2",
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  line_separator_trailing,
  // should match trailing line separator in text of list item
  "A:: a\nB:: b\u{2028}\nC:: c",
  contains: "<dd><p>b\u{2028}</p></dd>"
);

assert_html!(
  line_separator_within,
    // should match line separator in text of list item
  "A:: a\nB:: b\u{2028}b\nC:: c",
  contains: "<dd><p>b\u{2028}b</p></dd>"
);

assert_html!(
  rx_lists_tests_1,
  adoc! {r#"
    // should not parse a bare dlist delimiter as a dlist
    ::

    // should not parse an indented bare dlist delimiter as a dlist
     ::

    // missing space before term does not produce description list
    term1::def1
    term2::def2

    // should parse a dlist delimiter preceded by a blank attribute as a dlist
    {blank}::

    // should parse a dlist if term is include and principal text is []
    include:: []

    // should parse a dlist if term is include and principal text matches macro form
    include:: pass:[${placeholder}]

    // should parse sibling items using same rules
    term1;; ;; def1
    term2;; ;; def2

    // should allow term to end with a semicolon when using double semicolon delimiter
    term;;; def
  "#},
  html! {r#"
    <div class="paragraph"><p>::</p></div>
    <div class="literalblock">
      <div class="content"><pre>::</pre></div>
    </div>
    <div class="paragraph"><p>term1::def1 term2::def2</p></div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1"></dt>
        <dt class="hdlist1">include</dt>
        <dd><p>[]</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">include</dt>
        <dd><p>${placeholder}</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>;; def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>;; def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term;</dt>
        <dd><p>def</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  rx_lists_tests_2,
  adoc! {r#"
    // single-line indented adjacent elements
    term1:: def1
     term2:: def2

    // single-line elements separated by blank line should create a single list
    term1:: def1

    term2:: def2

    // a line comment between elements should divide them into separate lists
    term1:: def1

    //

    term2:: def2

    // a ruler between elements should divide them into separate lists
    term1:: def1

    '''

    term2:: def2

    // a block title between elements should divide them into separate lists
    term1:: def1

    .Some more
    term2:: def2
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
      </dl>
    </div>
    <hr>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <div class="title">Some more</div>
      <dl>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
  "#}
);

assert_html!(
  rx_lists_tests_3,
  adoc! {r#"
    // multi-line elements with paragraph content
    term1::
    def1
    term2::
    def2

    // multi-line elements with indented paragraph content
    term1::
     def1
    term2::
      def2

    // multi-line elements with blank line before paragraph content
    term3::

    def3
    term4::

    def4

    // mixed single and multi-line adjacent elements
    term5:: def5
    term6::
    def6
  "#},
  html! {r#"
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term1</dt>
        <dd><p>def1</p></dd>
        <dt class="hdlist1">term2</dt>
        <dd><p>def2</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term3</dt>
        <dd><p>def3</p></dd>
        <dt class="hdlist1">term4</dt>
        <dd><p>def4</p></dd>
      </dl>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1">term5</dt>
        <dd><p>def5</p></dd>
        <dt class="hdlist1">term6</dt>
        <dd><p>def6</p></dd>
      </dl>
    </div>
  "#}
);

// NB: asciicoctor does not render `[grays-peak]` as the link text,
// it uses the term text `Grays Peak`, for now we're not supporting this
// see `/todo.md#differences-from-asciidoctor`
assert_html!(
  anchors_starting_desc_terms,
  adoc! {r#"
    // should discover anchor at start of description term text and register it as a reference
    Highest is <<grays-peak>>, which tops <<mount-evans>>.

    [[mount-evans,Mount Evans]]Mount Evans:: 14,271 feet
    [[grays-peak]]Grays Peak:: 14,278 feet
  "#},
  html! {r##"
    <div class="paragraph">
      <p>Highest is <a href="#grays-peak">[grays-peak]</a>, which tops <a href="#mount-evans">Mount Evans</a>.</p>
    </div>
    <div class="dlist">
      <dl>
        <dt class="hdlist1"><a id="mount-evans"></a>Mount Evans</dt>
        <dd><p>14,271 feet</p></dd>
        <dt class="hdlist1"><a id="grays-peak"></a>Grays Peak</dt>
        <dd><p>14,278 feet</p></dd>
      </dl>
    </div>
  "##}
);

assert_html!(
  horizontal_desc_list,
  adoc! {r#"
    [horizontal.properties%step]
    property 1:: does stuff
    property 2:: does different stuff
  "#},
  html! {r##"
    <div class="hdlist properties">
      <table>
        <tr>
          <td class="hdlist1">property 1</td>
          <td class="hdlist2"><p>does stuff</p></td>
        </tr>
        <tr>
          <td class="hdlist1">property 2</td>
          <td class="hdlist2"><p>does different stuff</p></td>
        </tr>
      </table>
    </div>
  "##}
);

assert_html!(
  horizontal_desc_list_widths,
  adoc! {r#"
    [horizontal.properties%step,labelwidth=25,itemwidth="75%"]
    property 1:: does stuff
  "#},
  html! {r##"
    <div class="hdlist properties">
      <table>
        <colgroup>
          <col style="width: 25%;">
          <col style="width: 75%;">
        </colgroup>
        <tr>
          <td class="hdlist1">property 1</td>
          <td class="hdlist2"><p>does stuff</p></td>
        </tr>
      </table>
    </div>
  "##}
);

assert_html!(
  qanda_desc_list,
  adoc! {r#"
    [qanda]
    What is the answer?::
    This is the answer.

    Are cameras allowed?::
    Are backpacks allowed?::
    No.
  "#},
  html! {r##"
    <div class="qlist qanda">
      <ol>
        <li>
          <p><em>What is the answer?</em></p>
          <p>This is the answer.</p>
        </li>
        <li>
          <p><em>Are cameras allowed?</em></p>
          <p><em>Are backpacks allowed?</em></p>
          <p>No.</p>
        </li>
      </ol>
    </div>
  "##}
);
