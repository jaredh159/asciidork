use crate::{assert_asg_doc, assert_asg_inline};

assert_asg_inline!(
  inline_no_markup_single_word, //
  "inline/no-markup/single-word"
);

assert_asg_inline!(
  inline_span_strong_constrained_single_char,
  "inline/span/strong/constrained-single-char"
);

assert_asg_doc!(
  block_document_body_only, //
  "block/document/body-only"
);

assert_asg_doc!(
  block_document_header_body, //
  "block/document/header-body"
);

assert_asg_doc!(
  block_header_attribute_entries_below_title,
  "block/header/attribute-entries-below-title"
);

assert_asg_doc!(
  block_list_unordered_single_item,
  "block/list/unordered/single-item"
);

assert_asg_doc!(
  block_listing_multiple_lines, //
  "block/listing/multiple-lines"
);

assert_asg_doc!(
  block_paragraph_multiple_lines,
  "block/paragraph/multiple-lines"
);

assert_asg_doc!(
  block_paragraph_paragraph_empty_lines_paragraph,
  "block/paragraph/paragraph-empty-lines-paragraph"
);

assert_asg_doc!(
  block_paragraph_sibling_paragraps,
  "block/paragraph/sibling-paragraphs"
);

assert_asg_doc!(
  block_section_title_body, //
  "block/section/title-body"
);

assert_asg_doc!(
  block_sidebar_containing_unordered_list,
  "block/sidebar/containing-unordered-list"
);
