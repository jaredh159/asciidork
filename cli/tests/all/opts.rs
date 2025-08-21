use crate::helpers::*;

#[test]
fn attrs_defined_from_cli_opts() {
  let stdout = run_input(&["-a", "cliattr=asciidork", "-e"], "hello {cliattr}");
  assert!(stdout.contains("<p>hello asciidork</p>"));
}

#[test]
fn attrs_defined_from_cli_opts_subs() {
  // NB: this is different from rx cli, where subs are NOT applied to attributes
  // passed by the cli, but that feels like an leaking implementation detail
  // @see https://docs.asciidoctor.org/asciidoc/latest/attributes/attribute-entry-substitutions
  let stdout = run_input(&["-a", "cliattr=bat & ball", "-e"], "hello {cliattr}");
  assert!(stdout.contains("<p>hello bat &amp; ball</p>"));
}
