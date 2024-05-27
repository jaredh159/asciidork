## todo

- https://docs.asciidoctor.org/asciidoc/latest/attributes/assignment-precedence
- https://docs.asciidoctor.org/asciidoc/latest/attributes/document-attributes-ref

- [ ] resolving the {email} attr is example of "multiple passes", as it gets turned into
      an autolink in asciidoctor html5, test `{email}` vs `[subs=-macros]\n{email}`
- [ ] syntax for undefining an attribute: `{set:foo!}`, see
      https://docs.asciidoctor.org/asciidoc/latest/attributes/unresolved-references/#undefined
- [ ] think about this statement from header row section of tables docs: "Values assigned
      using the shorthand syntax must be entered before the cols attribute (or any other
      named attributes) in a table’s attribute list, otherwise the processor will ignore
      them."
- [ ] xref to discrete heading
- [ ] attr list `options` longhand (maybe role too?), i.e. `[options="a,b"]` == `[%a%b]`
- [ ] finish eval-ing all inline types, try to eval `kitchen-sink.adoc` to find missing
      ones
- [ ] finish fleshing out customizing substitutions, handling append/prepend, etc., see
      `customize_subs.rs`
- [ ] i don't run substitutions in an _order_. i need to search out some test cases of why
      (if?) this is naive/problematic, and fix
- [ ] section (and elsewhere?) auxiliary ids:
      https://docs.asciidoctor.org/asciidoc/latest/sections/custom-ids/#assign-auxiliary-ids
- [ ] work through this (and similar) lists, carefully adding/removing substutitions, with
      tests
- [ ] asciidoctor html backend _stylesheets,_ @see
      https://docs.asciidoctor.org/asciidoctor/latest/html-backend/stylesheet-modes/ and
      `html5.rb`
- [ ] char replacement substitutions:
      https://docs.asciidoctor.org/asciidoc/latest/subs/replacements/
- [ ] look into cleaning up errors with a macro `err!(tok_start: token, "foo")`
- [ ] all attribute refs, see
      https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/ bottom
      `{docdate}` example, see also
      https://docs.asciidoctor.org/asciidoc/latest/attributes/document-attributes-ref/#note-docdatetime
- [ ] h1 subtitle
- [√] indented listing/literal blocks
- [√] breaks
- [√] hard breaks
- [√] basic multi-byte char test
- [√] sections
- [√] poc, non-corner-painting wasm
- [√] description lists
- [√] multi-line continuation `\` -- see doc attrs, which can be multiline. prob should be
  folded into ContiguousLines so that it shows up as one line, i think
- [√] literal blocks (delimited and non-delimited)
- [√] listing blocks (delimited and non-delimited)
- [•] soon: multi-file non corner painting
- [√] block quotes
- [√] poc, non-corner-painting backend/interpreter
- [√] unsetting doc attrs, e.g. `:version-label!:`
- [√] contiguous sidebar with no delimiters using [sidebar]
- [√] whacky `.Optional title` block first line above blocks (search docs for "sidebar")
- [√] email autolinks:
- [√] need to track locations in nodes, like inline at least, maybe doc header
- [√] shared test macros (duplication of s!)
- [√] would be nice if it could report ALL parse errors (maybe sync on new block)
- [√] revision line for header
- [√] (tired) maybe move diagnostics into a RefCell, remove lots of mut parser

^ NB: commit `b035118` is useful if you want to find anything from the first, non
bump-allocated version

## questions

- listing/literal block indent method: is multiline supported? asciidoc does support, but
  it's not documented...
- do we track source locations for attr_lists?
- how can i see how asciidoctor emits asg?
- special char substitution... when? what do asg source locations look like for these?
- for a pass:[] macro, what does the asg want? do i just parse according to the indicated
  subs and discard the macro (wrt ast), or do i need to track the macro invocation holding
  a vec of inlines parsed according to subs?
- pass:[] macro docs contain a list of allowed substitution values, and then the example
  right below it shows using a value `q` not on the list!
- docs seems to say that a block _title_ needs to be _above_ the attr list, but dr. seems
  to parse it the same in either order
- https://docs.asciidoctor.org/asciidoc/latest/lists/ordered/#escaping-the-list-marker -
  shows how to escape `P. O. Box`, but I can't find any documentation on why `P.` should
  be considered a list marker - arbitrary letters don't show up anywhere in the ordered
  list documentation, and the current implementation doesn't get tripped up by
  `P. O. Box`, so I skipped this for now, until i can get some clarity from the
  asciidoctor team
- inside listing blocks, newlines are preserved, but DR. seems to trim leading/trailing
  newlines, and only honor those between, test "----\n\n\nfoo\n\nbar\n\n\n----", is this
  "spec", or just an accident/bug? (dork does not work this way currently)
- thematic break doesn't seem to support an attr list for adding classes to it? oddly,
  there's a test in asciidoctor showing its supported, but i can't seem to get it to work
  with the latest ruby cli

## weirdnesses...

- footnote:[] macro takes an attr list, but it seems like it only supports a single
  positional attribute
- "To insert an empty line somewhere in a paragraph, you can use the hard line break
  syntax (i.e., {empty}{plus}) on a line by itself. This allows you to insert space
  between lines in the output without introducing separate paragraphs." from
  `/hard-line-breaks` in docs, but i can't seem to replicate this behavior in
  asciidoctor...
