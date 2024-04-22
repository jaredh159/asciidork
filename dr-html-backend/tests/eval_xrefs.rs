use test_utils::*;

mod helpers;

test_eval_loose!(
  xrefs,
  adoc! {r#"
    == Tigers

    See <<_tigers>> for more information.

    This <<_ligers>> xref is broken.
  "#},
  html! {r##"
    <div class="sect1">
      <h2 id="_tigers">Tigers</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>See <a href="#_tigers">Tigers</a> for more information.</p>
        </div>
        <div class="paragraph">
          <p>This <a href="#_ligers">[_ligers]</a> xref is broken.</p>
        </div>
      </div>
    </div>
  "##}
);
