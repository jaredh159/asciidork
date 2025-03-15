#![macro_use]

#[macro_export]
macro_rules! assert_asg_doc {
  ($name:ident, $path:expr) => {
    #[test]
    fn $name() {
      use std::fs::read_to_string;
      let input = read_to_string(format!("tests/all/{}-input.adoc", $path)).unwrap();
      let asg = asciidork_tck::tck::gen_asg_doc(&input);
      println!("asg: {}", &asg);
      let asg: serde_json::Value = serde_json::from_str(&asg).unwrap();
      let asg = serde_json::to_string_pretty(&asg).unwrap();

      let expected = read_to_string(format!("tests/all/{}-output.json", $path)).unwrap();
      let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();
      let expected = serde_json::to_string_pretty(&expected).unwrap();

      ::pretty_assertions::assert_eq!(asg, expected);
    }
  };
}

#[macro_export]
macro_rules! assert_asg_inline {
  ($name:ident, $path:expr) => {
    #[test]
    fn $name() {
      use std::fs::read_to_string;
      let input = read_to_string(format!("tests/all/{}-input.adoc", $path)).unwrap();
      let asg = asciidork_tck::tck::gen_asg_inline(&input);
      println!("asg: {}", &asg);
      let asg: serde_json::Value = serde_json::from_str(&asg).unwrap();
      let asg = serde_json::to_string_pretty(&asg).unwrap();

      let expected = read_to_string(format!("tests/all/{}-output.json", $path)).unwrap();
      let expected: serde_json::Value = serde_json::from_str(&expected).unwrap();
      let expected = serde_json::to_string_pretty(&expected).unwrap();

      ::pretty_assertions::assert_eq!(asg, expected);
    }
  };
}
