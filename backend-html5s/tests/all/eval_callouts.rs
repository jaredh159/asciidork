use test_utils::*;

// NOTE: Callout tests use source helper functions that need updating for jirutka structure

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
  xml_callouts,
  adoc! {r#"
    [source,xml]
    ----
    <section>
      <title>Section Title</title> <!--1-->
    </section>
    ----
  "#},
  raw_html! {r#"
    <div class="listing-block"><pre class="highlight"><code class="language-xml" data-lang="xml">&lt;section&gt;
      &lt;title&gt;Section Title&lt;/title&gt; <b class="conum">1</b>
    &lt;/section&gt;</code></pre></div>"#}
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
  raw_html! {r#"
    <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">puts "1" <b class="conum">1</b>
    puts "2" <b class="conum">2</b></code></pre></div>"#}
);

// assert_html!(
//   callout_behind_comment,
//   adoc! {r#"
//     [source,ruby,line-comment=--]
//     ----
//     require 'sinatra' # <1>
//     require 'sinatra' // <2>
//     require 'sinatra' #<3>
//     require 'sinatra' -- <4>
//     require 'sinatra' --<5>
//     ----
//   "#},
//   raw_html! {r#"
//     <div class="listing-block"><pre class="highlight"><code class="language-ruby" data-lang="ruby">require 'sinatra' # <b class="conum">1</b>
//     require 'sinatra' // <b class="conum">2</b>
//     require 'sinatra' #<b class="conum">3</b>
//     require 'sinatra' <b class="conum">4</b>
//     require 'sinatra' <b class="conum">5</b></code></pre></div>"#}
// );
