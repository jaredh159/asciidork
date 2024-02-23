use asciidork_backend_asciidoctor_html::AsciidoctorHtml;
use asciidork_eval::{eval, Flags};
use asciidork_parser::prelude::*;

use indoc::indoc;
use pretty_assertions::assert_eq;
use regex::Regex;

#[test]
fn test_eval() {
  let cases = vec![
    (
      "_foo_\nbar\n\n",
      r#"<div class="paragraph"><p><em>foo</em> bar</p></div>"#,
    ),
    (
      "`*_foo_*`",
      r#"<div class="paragraph"><p><code><strong><em>foo</em></strong></code></p></div>"#,
    ),
    (
      "+_<foo>&_+",
      r#"<div class="paragraph"><p>_&lt;foo&gt;&amp;_</p></div>"#,
    ),
    (
      "foo #bar#",
      r#"<div class="paragraph"><p>foo <mark>bar</mark></p></div>"#,
    ),
    (
      "foo `bar`",
      r#"<div class="paragraph"><p>foo <code>bar</code></p></div>"#,
    ),
    (
      "rofl +_foo_+ lol",
      r#"<div class="paragraph"><p>rofl _foo_ lol</p></div>"#,
    ),
    (
      "+++_<foo>&_+++ bar",
      r#"<div class="paragraph"><p>_<foo>&_ bar</p></div>"#,
    ),
    (
      "foo ~bar~ baz",
      r#"<div class="paragraph"><p>foo <sub>bar</sub> baz</p></div>"#,
    ),
    (
      "foo ^bar^ baz",
      r#"<div class="paragraph"><p>foo <sup>bar</sup> baz</p></div>"#,
    ),
    (
      "foo `'bar'`",
      r#"<div class="paragraph"><p>foo <code>'bar'</code></p></div>"#,
    ),
    (
      "foo \"`bar`\"",
      r#"<div class="paragraph"><p>foo &#8220;bar&#8221;</p></div>"#,
    ),
    (
      "Olaf's wrench",
      r#"<div class="paragraph"><p>Olaf&#8217;s wrench</p></div>"#,
    ),
    (
      "foo   bar",
      r#"<div class="paragraph"><p>foo bar</p></div>"#,
    ),
    (
      "`+{name}+`",
      r#"<div class="paragraph"><p><code>{name}</code></p></div>"#,
    ),
    (
      "foo <bar> & lol",
      r#"<div class="paragraph"><p>foo &lt;bar&gt; &amp; lol</p></div>"#,
    ),
    (
      "press the btn:[OK] button",
      r#"<div class="paragraph"><p>press the <b class="button">OK</b> button</p></div>"#,
    ),
    (
      "select menu:File[Save].",
      indoc! {r#"
        <div class="paragraph">
          <p>select <span class="menuseq"><span class="menu">File</span>&#160;&#9656;<span class="menuitem">Save</span></span>.</p>
        </div>
      "#},
    ),
    (
      "select menu:File[Save > Reset].",
      indoc! {r#"
        <div class="paragraph">
          <p>
            select <span class="menuseq"
              ><span class="menu">File</span>&#160;&#9656;
              <span class="submenu">Save</span>&#160;&#9656;
              <span class="menuitem">Reset</span></span
            >.
          </p>
        </div>
      "#},
    ),
    (
      "[sidebar]\nfoo bar",
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            foo bar
          </div>
        </div>
      "#},
    ),
    (
      ".Title\nfoo",
      indoc! {r#"
        <div class="paragraph">
          <div class="title">Title</div>
          <p>foo</p>
        </div>
      "#},
    ),
    (
      "--\nfoo\n--",
      indoc! {r#"
        <div class="openblock">
          <div class="content">
            <div class="paragraph">
              <p>foo</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      "====\nfoo\n====",
      indoc! {r#"
        <div class="exampleblock">
          <div class="content">
            <div class="paragraph">
              <p>foo</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      "[#my-id.some-class]\nTIP: never start a land war in Asia",
      indoc! {r#"
        <div id="my-id" class="admonitionblock tip some-class">
          <table>
            <tr>
              <td class="icon">
                <div class="title">Tip</div>
              </td>
              <td class="content">
                never start a land war in Asia
              </td>
            </tr>
          </table>
        </div>
      "#},
    ),
    (
      ".Title\nNOTE: foo",
      indoc! {r#"
        <div class="admonitionblock note">
          <table>
            <tr>
              <td class="icon">
                <div class="title">Note</div>
              </td>
              <td class="content">
                <div class="title">Title</div>
                foo
              </td>
            </tr>
          </table>
        </div>
      "#},
    ),
    (
      "image::name.png[]",
      indoc! {r#"
        <div class="imageblock">
          <div class="content">
            <img src="name.png" alt="name">
          </div>
        </div>
      "#},
    ),
    (
      ".Title\n[#lol.rofl]\nimage::cat.jpg[]",
      indoc! {r#"
        <div id="lol" class="imageblock rofl">
          <div class="content">
            <img src="cat.jpg" alt="cat">
          </div>
          <div class="title">Figure 1. Title</div>
        </div>
      "#},
    ),
    (
      "[quote,,cite]\nfoo bar",
      indoc! {r#"
        <div class="quoteblock">
          <blockquote>
            foo bar
          </blockquote>
          <div class="attribution">
            &#8212; cite
          </div>
        </div>
      "#},
    ),
    (
      "[quote,source]\nfoo bar",
      indoc! {r#"
        <div class="quoteblock">
          <blockquote>
            foo bar
          </blockquote>
          <div class="attribution">
            &#8212; source
          </div>
        </div>
      "#},
    ),
    (
      "[quote,source,location]\nfoo bar",
      indoc! {r#"
        <div class="quoteblock">
          <blockquote>
            foo bar
          </blockquote>
          <div class="attribution">
            &#8212; source<br>
            <cite>location</cite>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .After landing the cloaked Klingon bird of prey in Golden Gate park:
        [quote,Captain James T. Kirk,Star Trek IV: The Voyage Home]
        Everybody remember where we parked.
      "#},
      indoc! {r#"
        <div class="quoteblock">
          <div class="title">After landing the cloaked Klingon bird of prey in Golden Gate park:</div>
          <blockquote>
            Everybody remember where we parked.
          </blockquote>
          <div class="attribution">
            &#8212; Captain James T. Kirk<br>
            <cite>Star Trek IV: The Voyage Home</cite>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        [quote,Monty Python and the Holy Grail]
        ____
        Dennis: Come and see the violence inherent in the system. Help! Help!

        King Arthur: Bloody peasant!
        ____
      "#},
      indoc! {r#"
        <div class="quoteblock">
          <blockquote>
            <div class="paragraph">
              <p>Dennis: Come and see the violence inherent in the system. Help! Help!</p>
            </div>
            <div class="paragraph">
              <p>King Arthur: Bloody peasant!</p>
            </div>
          </blockquote>
          <div class="attribution">
            &#8212; Monty Python and the Holy Grail
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        "I hold it that a little rebellion now and then is a good thing,
        and as necessary in the political world as storms in the physical."
        -- Thomas Jefferson, Papers of Thomas Jefferson: Volume 11
      "#},
      indoc! {r#"
        <div class="quoteblock">
          <blockquote>
            I hold it that a little rebellion now and then is a good thing, and as necessary in the political world as storms in the physical.
          </blockquote>
          <div class="attribution">
            &#8212; Thomas Jefferson<br>
            <cite>Papers of Thomas Jefferson: Volume 11</cite>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        ****
        --
        foo
        --
        ****
      "#},
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            <div class="openblock">
              <div class="content">
                <div class="paragraph">
                  <p>foo</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .Cat
        image::cat.png[]

        .Dog
        image::dog.png[]
      "#},
      indoc! {r#"
        <div class="imageblock">
          <div class="content">
            <img src="cat.png" alt="cat">
          </div>
          <div class="title">Figure 1. Cat</div>
        </div>
        <div class="imageblock">
          <div class="content">
            <img src="dog.png" alt="dog">
          </div>
          <div class="title">Figure 2. Dog</div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        = Doc Header
        :!figure-caption:

        .Cat
        image::cat.png[]

        .Dog
        image::dog.png[]
      "#},
      indoc! {r#"
        <div class="imageblock">
          <div class="content">
            <img src="cat.png" alt="cat">
          </div>
          <div class="title">Cat</div>
        </div>
        <div class="imageblock">
          <div class="content">
            <img src="dog.png" alt="dog">
          </div>
          <div class="title">Dog</div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .A mountain sunset
        [#img-sunset,link=https://www.flickr.com/photos/javh/5448336655]
        image::sunset.jpg[Sunset,200,100]
      "#},
      indoc! {r#"
        <div id="img-sunset" class="imageblock">
          <div class="content">
            <a class="image" href="https://www.flickr.com/photos/javh/5448336655">
              <img src="sunset.jpg" alt="Sunset" width="200" height="100">
            </a>
          </div>
          <div class="title">Figure 1. A mountain sunset</div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        .Title
        image::foo.png[]

        :!figure-caption:

        .Next
        image::bar.png[]
      "#},
      indoc! {r#"
        <div class="imageblock">
          <div class="content">
            <img src="foo.png" alt="foo">
          </div>
          <div class="title">Figure 1. Title</div>
        </div>
        <div class="imageblock">
          <div class="content">
            <img src="bar.png" alt="bar">
          </div>
          <div class="title">Next</div>
        </div>
      "#},
    ),
    (
      "foo.footnote:[bar _baz_]",
      indoc! {r##"
        <div class="paragraph">
          <p>foo.
            <sup class="footnote">
              [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
            </sup>
          </p>
        </div>
        <div id="footnotes">
          <hr>
          <div class="footnote" id="_footnotedef_1">
            <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
          </div>
        </div>
      "##},
    ),
    (
      indoc! {r#"
        ****
        This is content in a sidebar block.

        image::name.png[]

        This is more content in the sidebar block.
        ****
      "#},
      indoc! {r#"
        <div class="sidebarblock">
          <div class="content">
            <div class="paragraph">
              <p>This is content in a sidebar block.</p>
            </div>
            <div class="imageblock">
              <div class="content">
                <img src="name.png" alt="name">
              </div>
            </div>
            <div class="paragraph">
              <p>This is more content in the sidebar block.</p>
            </div>
          </div>
        </div>
      "#},
    ),
    (
      indoc! {r#"
        foo.footnote:[bar _baz_]

        lol.footnote:cust[baz]
      "#},
      indoc! {r##"
      <div class="paragraph">
        <p>foo.
          <sup class="footnote">
            [<a id="_footnoteref_1" class="footnote" href="#_footnotedef_1" title="View footnote.">1</a>]
          </sup>
        </p>
      </div>
      <div class="paragraph">
        <p>lol.
          <sup class="footnote" id="_footnote_cust">
            [<a id="_footnoteref_2" class="footnote" href="#_footnotedef_2" title="View footnote.">2</a>]
          </sup>
        </p>
      </div>
      <div id="footnotes">
        <hr>
        <div class="footnote" id="_footnotedef_1">
          <a href="#_footnoteref_1">1</a>. bar <em>baz</em>
        </div>
        <div class="footnote" id="_footnotedef_2">
          <a href="#_footnoteref_2">2</a>. baz
        </div>
      </div>
    "##},
    ),
    (
      "* foo\n* bar",
      indoc! {r#"
        <div class="ulist">
          <ul>
            <li><p>foo</p></li>
            <li><p>bar</p></li>
          </ul>
        </div>
      "#},
    ),
    (
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
      "#},
    ),
    (
      indoc! {r#"
        * foo _bar_
          so *baz*
        * two
      "#},
      indoc! {r#"
        <div class="ulist">
          <ul>
            <li>
              <p>foo <em>bar</em> so <strong>baz</strong></p>
            </li>
            <li>
              <p>two</p>
            </li>
          </ul>
        </div>
      "#},
    ),
    (
      "* foo\n** bar",
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
      "#},
    ),
    (
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
       "#},
    ),
    (
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
      "#},
    ),
  ];
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  for (input, expected) in cases {
    let expected = re.replace_all(expected, "");
    let parser = Parser::new(bump, input);
    let doc = parser.parse().unwrap().document;
    // dbg!(&doc);
    assert_eq!(
      eval(doc, Flags::embedded(), AsciidoctorHtml::new()).unwrap(),
      expected,
      "input was\n\n{}",
      input
    );
  }
}

enum SubstrTest {
  Contains(&'static str),
  DoesNotContain(&'static str),
}

#[test]
fn test_head_opts() {
  use SubstrTest::*;
  let cases = vec![
    (":nolang:", DoesNotContain("lang=")),
    (":nolang:", Contains("<title>Doc Header</title>")),
    (
      ":title: Such Custom Title",
      Contains("<title>Such Custom Title</title>"),
    ),
    (":lang: es", Contains("lang=\"es\"")),
    (":encoding: latin1", Contains("charset=\"latin1\"")),
    (":reproducible:", DoesNotContain("generator")),
    (
      ":app-name: x",
      Contains(r#"<meta name="application-name" content="x">"#),
    ),
    (
      ":description: x",
      Contains(r#"<meta name="description" content="x">"#),
    ),
    (
      ":keywords: x, y",
      Contains(r#"<meta name="keywords" content="x, y">"#),
    ),
    (
      "Kismet R. Lee <kismet@asciidoctor.org>",
      Contains(r#"<meta name="author" content="Kismet R. Lee">"#),
    ),
    (
      "Kismet R. Lee <kismet@asciidoctor.org>; Bob Smith",
      Contains(r#"<meta name="author" content="Kismet R. Lee, Bob Smith">"#),
    ),
    (
      ":copyright: x",
      Contains(r#"<meta name="copyright" content="x">"#),
    ),
    (
      ":favicon:",
      Contains(r#"<link rel="icon" type="image/x-icon" href="favicon.ico">"#),
    ),
    (
      ":favicon: ./images/favicon/favicon.png",
      Contains(r#"<link rel="icon" type="image/png" href="./images/favicon/favicon.png">"#),
    ),
    (
      ":iconsdir: custom\n:favicon: {iconsdir}/my/icon.png",
      Contains(r#"<link rel="icon" type="image/png" href="custom/my/icon.png">"#),
    ),
  ];
  let bump = &Bump::new();

  for (opts, expectation) in cases {
    let input = format!("= Doc Header\n{}\n\nignore me\n\n", opts);
    let parser = Parser::new(bump, &input);
    let document = parser.parse().unwrap().document;
    let html = eval(document, Flags::default(), AsciidoctorHtml::new()).unwrap();
    match expectation {
      Contains(s) => assert!(
        html.contains(s),
        "\n`{}` was NOT found when expected\n\n```adoc\n{}\n```\n\n```html\n{}\n```",
        s,
        input.trim(),
        html.replace('>', ">\n").trim()
      ),
      DoesNotContain(s) => assert!(
        !html.contains(s),
        "\n`{}` WAS found when not expected\n\n```adoc\n{}\n```\n\n```html\n{}\n```",
        s,
        input.trim(),
        html.replace('>', ">\n").trim()
      ),
    }
  }
  // one test with no doc header
  let parser = Parser::new(bump, "without doc header");
  let document = parser.parse().unwrap().document;
  let html = eval(document, Flags::default(), AsciidoctorHtml::new()).unwrap();
  assert!(html.contains("<title>Untitled</title>"));
}

#[test]
fn test_non_embedded() {
  let input = indoc! {r#"
    = *Document* _title_

    foo
  "#};
  let expected = indoc! {r##"
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="generator" content="Asciidork">
        <title>Document title</title>
      </head>
      <body>
        <div class="paragraph">
          <p>foo</p>
        </div>
      </body>
    </html>
  "##};
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let parser = Parser::new(bump, input);
  let doc = parser.parse().unwrap().document;
  assert_eq!(
    eval(doc, Flags::default(), AsciidoctorHtml::new()).unwrap(),
    expected,
    "input was {}",
    input
  );
}

#[test]
fn test_isolate() {
  let input = indoc! {r#"
    foo
  "#};
  let expected = indoc! {r##"
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <meta http-equiv="X-UA-Compatible" content="IE=edge">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <meta name="generator" content="Asciidork">
        <title>Untitled</title>
      </head>
      <body>
        <div class="paragraph">
          <p>foo</p>
        </div>
      </body>
    </html>
  "##};
  let bump = &Bump::new();
  let re = Regex::new(r"(?m)\n\s*").unwrap();
  let expected = re.replace_all(expected, "");
  let parser = Parser::new(bump, input);
  let doc = parser.parse().unwrap().document;
  assert_eq!(
    eval(doc, Flags::default(), AsciidoctorHtml::new()).unwrap(),
    expected,
    "input was {}",
    input
  );
}
