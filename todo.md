## todo

- [ ] char replacement substitutions:
      https://docs.asciidoctor.org/asciidoc/latest/subs/replacements/
- [ ] explore whether adding `std` to bumpalo gives more stuff for file conversion, etc
- [ ] look into cleaning up errors with a macro `err!(tok_start: token, "foo")`
- [ ] basic multi-byte char test
- [ ] poc, non-corner-painting backend/interpreter
- [ ] poc, non-corner-painting wasm
- [ ] attribute refs, see
      https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/ bottom
      `{docdate}` example
- [ ] h1 subtitle
      https://docs.asciidoctor.org/asciidoc/latest/macros/autolinks/#email-autolinks
- [ ] unsetting doc attrs, e.g. `:version-label!:`
- [•] soon: multi-file non corner painting
- [√] email autolinks:
- [√] need to track locations in nodes, like inline at least, maybe doc header
- [√] shared test macros (duplication of s!)
- [√] would be nice if it could report ALL parse errors (maybe sync on new block)
- [√] revision line for header
- [√] (tired) maybe move diagnostics into a RefCell, remove lots of mut parser

^ NB: commit `b035118` is useful if you want to find anything from the first, non
bump-allocated version

## questions

- do we track source locations for attr_lists?
- how can i see how asciidoctor emits asg?
- special char substitution... when? what do asg source locations look like for these?
- for a pass:[] macro, what does the asg want? do i just parse according to the indicated
  subs and discard the macro (wrt ast), or do i need to track the macro invocation holding
  a vec of inlines parsed according to subs?
- pass:[] macro docs contain a list of allowed substitution values, and then the example
  right below it shows using a value `q` not on the list!

## weirdnesses...

- footnote:[] macro takes an attr list, but it seems like it only supports a single
  positional attribute
