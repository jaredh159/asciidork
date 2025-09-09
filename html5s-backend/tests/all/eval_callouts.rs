use crate::helpers::source;
use test_utils::{adoc, raw_html};

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
  source::wrap(
    "ruby",
    raw_html! {r#"
      require 'sinatra' <b class="conum">(1)</b>

      get '/hi' do <b class="conum">(2)</b> <b class="conum">(3)</b>
        "Hello World!"
      end
    "#}
  )
);

assert_html!(
  xml_callouts,
  adoc! {r#"
    [source,xml]
    ----
    <section>
      <title>Section Title</title> <!--1-->
    </section>
    ----
  "#},
  source::wrap(
    "xml",
    raw_html! {r#"
      &lt;section&gt;
        &lt;title&gt;Section Title&lt;/title&gt; <b class="conum">(1)</b>
      &lt;/section&gt;
    "#}
  )
);

assert_html!(
  callouts_w_icons,
  adoc! {r#"
    :icons: font

    [source,ruby]
    ----
    puts "1" <1>
    puts "2" # <2>
    ----
  "#},
  source::wrap(
    "ruby",
    raw_html! {r#"
      puts "1" <i class="conum" data-value="1"></i><b>(1)</b>
      puts "2" <i class="conum" data-value="2"></i><b>(2)</b>
    "#}
  )
);

assert_html!(
  callout_behind_comment,
  adoc! {r#"
    [source,ruby,line-comment=--]
    ----
    require 'sinatra' # <1>
    require 'sinatra' // <2>
    require 'sinatra' #<3>
    require 'sinatra' -- <4>
    require 'sinatra' --<5>
    ----
  "#},
  source::wrap(
    "ruby",
    raw_html! {r#"
      require 'sinatra' # <b class="conum">(1)</b>
      require 'sinatra' // <b class="conum">(2)</b>
      require 'sinatra' # <b class="conum">(3)</b>
      require 'sinatra' -- <b class="conum">(4)</b>
      require 'sinatra' -- <b class="conum">(5)</b>
    "#}
  )
);
