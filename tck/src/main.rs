use std::io::Read;

use asciidork_tck::tck;

fn main() {
  let mut input = String::new();
  std::io::stdin().read_to_string(&mut input).unwrap();

  // https://gitlab.eclipse.org/eclipse/asciidoc-lang/asciidoc-tck/-/merge_requests/26/diffs
  let input: serde_json::Value = serde_json::from_str(&input).unwrap();
  let contents = input["contents"].as_str().unwrap();
  let input_type = input["type"].as_str().unwrap();

  let asg_json = if input_type == "inline" {
    tck::gen_asg_inline(contents)
  } else {
    tck::gen_asg_doc(contents)
  };

  println!("{asg_json}");
}
