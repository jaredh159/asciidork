## todo

- [ ] look into cleaning up errors with a macro `err!(tok_start: token, "foo")`
- [ ] basic multi-byte char test
- [ ] poc, non-corner-painting backend/interpreter
- [ ] poc, non-corner-painting wasm
- [ ] attribute refs, see
      https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/ bottom
      `{docdate}` example
- [ ] h1 subtitle
- [ ] email autolinks:
      https://docs.asciidoctor.org/asciidoc/latest/macros/autolinks/#email-autolinks
- [ ] unsetting doc attrs, e.g. `:version-label!:`
- [•] soon: multi-file non corner painting
- [√] would be nice if it could report ALL parse errors (maybe sync on new block)
- [√] revision line for header
- [√] (tired) maybe move diagnostics into a RefCell, remove lots of mut parser
