## todo

- https://docs.asciidoctor.org/asciidoc/latest/attributes/assignment-precedence
- https://docs.asciidoctor.org/asciidoc/latest/attributes/document-attributes-ref

- [ ] "natural" xrefs (eg: `<<Section Title>>`), these are officially discouraged, so i'd
      like to not implement them
- [ ] multi-line inline (not block) img macro attrs (search rx for `a multi-line image`)
- [ ] multi-line link macro text (maybe same as above)
      `link:https://example.com[foo\nbar]`
- [ ] multi-line shorthand xref `<<tigers,foo\nbar>>`
- [ ] data-uri embedded images
- [ ] "embedded" inline svgs, where html is in output, changing size
- [ ] index: https://docs.asciidoctor.org/asciidoc/latest/sections/user-index
- [ ] multi-anchors, e.g. `=== [[current]][[latest]]Version 4.9`, see
      https://docs.asciidoctor.org/asciidoc/latest/attributes/id/#add-additional-anchors-to-a-section
- [ ] resolve include directives starting from stdin
- [ ] rest of doc-attrs-ref, date stuff, output file things
- [ ] syntax for undefining an attribute: `{set:foo!}`, see
      https://docs.asciidoctor.org/asciidoc/latest/attributes/unresolved-references/#undefined
- [ ] think about this statement from header row section of tables docs: "Values assigned
      using the shorthand syntax must be entered before the cols attribute (or any other
      named attributes) in a table’s attribute list, otherwise the processor will ignore
      them."
- [ ] xref to discrete heading
- [ ] finish eval-ing all inline types, try to eval `kitchen-sink.adoc` to find missing
      ones
- [ ] finish fleshing out customizing substitutions, handling append/prepend, etc., see
      `customize_subs.rs`
- [ ] i don't run substitutions in an _order_. i need to search out some test cases of why
      (if?) this is naive/problematic, and fix (see inferred_doc_title_attr test)
- [ ] resolving the {email} attr is example of "multiple passes", as it gets turned into
      an autolink in asciidoctor html5, test `{email}` vs `[subs=-macros]\n{email}`
- [ ] section (and elsewhere?) auxiliary ids:
      https://docs.asciidoctor.org/asciidoc/latest/sections/custom-ids/#assign-auxiliary-ids
- [ ] work through this (and similar) lists, carefully adding/removing substutitions, with
      tests
- [ ] asciidoctor html backend _stylesheets,_ @see
      https://docs.asciidoctor.org/asciidoctor/latest/html-backend/stylesheet-modes/ and
      `html5.rb`
- [ ] all attribute refs, see
      https://docs.asciidoctor.org/asciidoc/latest/document/revision-line/ bottom
      `{docdate}` example, see also
      https://docs.asciidoctor.org/asciidoc/latest/attributes/document-attributes-ref/#note-docdatetime
- [ ] h1 subtitle
- [ ] `tabsize`, see
      https://docs.asciidoctor.org/asciidoc/latest/directives/include-with-indent/#the-indent-attribute
- [ ] doctype=book, special rules
- [ ] asciidoctor seems to resolve attr refs case-insensitive, grep `ifdef::showScript[]`
      (is it only for ifdef?) - not sure i want to replicate this, seems undocumented...

## differences from asciidoctor

- asciidoctor uses term of description list as reftext for a preceding anchor without
  reftext, so `[[foo]]Bar:: baz` produces an anchor with the refext of `Bar`. Currently we
  don't support this, the author would need to specify the reftext explicitly with
  `[[foo,Bar]]Bar:: baz`
- asciidoctor allows unnattached block attr lines, like `[[foo]]\n\n`, and seems to attach
  it to the next block. the documentation says there should be no empty line. asciidork
  sometimes will attach the metadata, and sometimes will not, but with `--strict` will
  always emit an error.

## design philosophy

- push complexity into the lexer, w/ more tokens, we can always ignore token types later
  based on substitutions
- try to keep things single-pass as much as possible, even if semantically the language
  describes things as having multiple ordered passes
- use regex as a last resort

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
- asciidork parses monos in both of the following lines, but asciidoc doesn't because of
  it's regex-based approach, but per the discussion on zulip, our approach is preferred:

```adoc
// @see https://asciidoc.zulipchat.com/#narrow/channel/335214-general
foo `bar`"
"foo `bar`"
```
