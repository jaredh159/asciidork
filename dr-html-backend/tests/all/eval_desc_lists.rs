use test_utils::{adoc, html};

assert_html!(
  simple_description_list,
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
  rx_lists_tests_1,
  adoc! {r#"
    // should not parse a bare dlist delimiter as a dlist
    ::

    // should not parse an indented bare dlist delimiter as a dlist
     ::

    // should parse a dlist delimiter preceded by a blank attribute as a dlist
    {blank}::

    // should parse a dlist if term is include and principal text is []
    include:: []

    // should parse a dlist if term is include and principal text matches macro form
    include:: pass:[${placeholder}]
  "#},
  html! {r#"
    <div class="paragraph"><p>::</p></div>
    <div class="literalblock">
      <div class="content"><pre>::</pre></div>
    </div>
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
