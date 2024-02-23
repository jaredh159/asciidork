use asciidork_backend_asciidoctor_html::AsciidoctorHtml;
use asciidork_eval::{eval, Flags};
use asciidork_parser::prelude::*;

use indoc::indoc;
use pretty_assertions::assert_eq;
use regex::Regex;

test_eval!(
  most_basic_unordered_list,
  indoc! {r#"
    * foo
    * bar
  "#},
  indoc! {r#"
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
  indoc! {r#"
    * Apples
    * Oranges

    //-

    * Walnuts
    * Almonds
  "#},
  indoc! {r#"
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
  indoc! {r#"
    * foo _bar_
      so *baz*
    * two
  "#},
  indoc! {r#"
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
  indoc! {r#"
    * foo
    ** bar
  "#},
  indoc! {r#"
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
  indoc! {r#"
    - Edgar Allan Poe
    - Sheri S. Tepper
    - Bill Bryson
  "#},
  indoc! {r#"
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
  indoc! {r#"
    .Kizmets Favorite Authors
    * Edgar Allan Poe
    * Sheri S. Tepper
    * Bill Bryson
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [square]
    * Level 1 list item
    - Level 2 list item
    * Level 1 list item
  "#},
  indoc! {r#"
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
  indoc! {r#"
    .Possible DefOps manual locations
    * West wood maze
    ** Maze heart
    *** Reflection pool
    ** Secret exit
    * Untracked file in git repository
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [square]
    * squares
    ** up top
    [circle]
    *** circles
    **** down below
  "#},
  indoc! {r#"
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
  indoc! {r#"
    1. Protons
    2. Electrons
    3. Neutrons
  "#},
  indoc! {r#"
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
  indoc! {r#"
    4. Protons
    5. Electrons
    6. Neutrons
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [start=4]
    . Protons
    . Electrons
    . Neutrons
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [%reversed]
    .Parts of an atom
    . Protons
    . Electrons
    . Neutrons
  "#},
  indoc! {r#"
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
  indoc! {r#"
    . Step 1
    . Step 2
    .. Step 2a
    .. Step 2b
    . Step 3
  "#},
  indoc! {r#"
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
  indoc! {r#"
    . Linux
    * Fedora
    * Ubuntu
    * Slackware
    . BSD
    * FreeBSD
    * NetBSD
  "#},
  indoc! {r#"
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
  indoc! {r#"
    . Linux

      * Fedora
      * Ubuntu
      * Slackware

    . BSD

      * FreeBSD
      * NetBSD
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [lowerroman,start=5]
    . Five
    . Six
    [loweralpha]
    .. a
    .. b
    .. c
    . Seven
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [.custom-class]
    * [*] checked
    * [x] also checked
    * [ ] not checked
    * normal list item
  "#},
  indoc! {r#"
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
  indoc! {r#"
    [%interactive]
    * [*] checked
    * [x] also checked
    * [ ] not checked
  "#},
  indoc! {r#"
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
  indoc! {r#"
    . [*] checked
    . [ ] not checked
  "#},
  indoc! {r#"
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
  indoc! {r#"
    * principle
    +
    with continuation
  "#},
  indoc! {r#"
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
  indoc! {r#"
    * principle
    +
    with continuation
    +
    and another
  "#},
  indoc! {r#"
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
  list_items_w_delimited_blocks,
  indoc! {r#"
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
  indoc! {r#"
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
  indoc! {r#"
     . {empty}
     +
     --
     para
     --
   "#},
  indoc! {r#"
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
  indoc! {r#"
    para

    :foo: bar

    . {foo}
  "#},
  indoc! {r#"
    <div class="paragraph"><p>para</p></div>
    <div class="olist arabic">
      <ol class="arabic">
        <li><p>bar</p></li>
      </ol>
    </div>
  "#}
);

// helpers

#[macro_export]
macro_rules! test_eval {
  ($name:ident, $input:expr, $expected:expr) => {
    #[test]
    fn $name() {
      let bump = &Bump::new();
      let re = Regex::new(r"(?m)\n\s*").unwrap();
      let expected = re.replace_all($expected, "");
      let parser = Parser::new(bump, $input);
      let doc = parser.parse().unwrap().document;
      assert_eq!(
        eval(doc, Flags::embedded(), AsciidoctorHtml::new()).unwrap(),
        expected,
        "input was\n\n{}",
        $input
      );
    }
  };
}
