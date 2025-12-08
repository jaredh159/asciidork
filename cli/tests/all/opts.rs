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

// https://docs.asciidoctor.org/asciidoc/latest/attributes/assignment-precedence/

#[test]
fn attr_precedence_cli_beats_document() {
  let input = ":myattr: from-document\n\n{myattr}";
  let stdout = run_input(&["-a", "myattr=from-cli", "-e"], input);
  assert!(stdout.contains("<p>from-cli</p>"));
}

#[test]
fn attr_precedence_document_beats_default() {
  // sectids is enabled by default, here document unsets it
  let input = ":!sectids:\n\n== Section Title\n\nContent";
  let stdout = run_input(&["-e"], input);
  assert!(!stdout.contains("id=\"_section_title\""));
}

#[test]
fn attr_precedence_cli_beats_default() {
  let input = "== Section Title\n\nContent";
  let stdout = run_input(&["-a", "!sectids", "-e"], input);
  assert!(!stdout.contains("id=\"_section_title\""));
  let stdout = run_input(&["-a", "sectids!", "-e"], input);
  assert!(!stdout.contains("id=\"_section_title\""));
}

#[test]
fn attr_precedence_document_beats_cli_soft() {
  let input = ":myattr: from-document\n\n{myattr}";
  let stdout = run_input(&["-a", "myattr=from-cli@", "-e"], input);
  assert!(stdout.contains("<p>from-document</p>"));
  let stdout = run_input(&["-a", "@myattr=from-cli", "-e"], input);
  assert!(stdout.contains("<p>from-document</p>"));
}

#[test]
fn attr_precedence_cli_unset_soft_beats_default() {
  let input = "== Section Title\n\nContent";
  let stdout = run_input(&["-a", "!sectids@", "-e"], input);
  assert!(!stdout.contains("id=\"_section_title\""));
}

#[test]
fn attr_precedence_cli_hard_set_beats_document_override() {
  let input = ":myattr: override-attempt\n\n{myattr}";
  let stdout = run_input(&["-a", "myattr=locked", "-e"], input);
  assert!(stdout.contains("<p>locked</p>"));
  assert!(!stdout.contains("override-attempt"));
}

#[test]
fn attr_precedence_soft_unset_allows_document_override() {
  let input = ":sectids:\n\n== Section Title\n\nContent";
  let stdout = run_input(&["-a", "!sectids@", "-e"], input);
  assert!(stdout.contains("id=\"_section_title\""));
}
