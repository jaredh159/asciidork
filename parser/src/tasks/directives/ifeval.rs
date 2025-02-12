use crate::internal::*;

impl<'arena> Parser<'arena> {
  pub(crate) fn try_process_ifeval_directive(
    &mut self,
    line: &mut Line<'arena>,
  ) -> Result<DirectiveAction<'arena>> {
    let src = line.reassemble_src();
    let Some(captures) = regx::DIRECTIVE_IFEVAL.captures(&src) else {
      if let Some(err_cap) = regx::DIRECTIVE_INVALID_IFEVAL.captures(&src) {
        let mut pattern = &err_cap[2];
        if pattern.is_empty() {
          pattern = &err_cap[1];
        }
        self.err_at_pattern(
          "Invalid ifeval directive expression",
          line.loc().unwrap(),
          pattern,
        )?;
      }
      return Ok(DirectiveAction::Passthrough);
    };

    if !&captures[1].is_empty() {
      self.err_at_pattern(
        "ifeval directive may not include a target",
        line.loc().unwrap(),
        &captures[1],
      )?;
      return Ok(DirectiveAction::Passthrough);
    }

    let op = &captures[3];
    let Some(op) = Op::from_str(op) else {
      return Ok(DirectiveAction::Passthrough);
    };

    let lhs = self.coerce_eval_expr(&captures[2]);
    let rhs = self.coerce_eval_expr(&captures[4]);
    if eval(lhs, op, rhs) {
      self.ctx.ifdef_stack.push(self.string("[•ifeval•]"));
      Ok(DirectiveAction::ReadNextLine)
    } else {
      Ok(DirectiveAction::SkipLinesUntilEndIf)
    }
  }

  fn coerce_eval_expr(&self, expr: &str) -> Value<'arena> {
    let expr = self.replace_attr_vals(expr);
    match &*expr {
      "true" => return Value::Boolean(true),
      "false" => return Value::Boolean(false),
      "" => return Value::Nil,
      _ => {}
    }
    if let Ok(int) = expr.parse::<isize>() {
      return Value::Int(int);
    } else if let Ok(float) = expr.parse::<f32>() {
      return Value::Float(float);
    }
    if expr.starts_with('\'') && expr.ends_with('\'')
      || expr.starts_with('"') && expr.ends_with('"')
    {
      Value::String(self.string(&expr[1..expr.len() - 1]))
    } else if expr.chars().all(|c| c.is_ascii_whitespace()) {
      Value::String(self.string(" "))
    } else {
      Value::Int(0)
    }
  }
}

fn eval(lhs: Value, op: Op, rhs: Value) -> bool {
  match (lhs, op, rhs) {
    // string
    (Value::String(lhs), Op::Eq, Value::String(rhs)) => lhs == rhs,
    (Value::String(lhs), Op::NotEq, Value::String(rhs)) => lhs != rhs,
    (Value::String(lhs), Op::Less, Value::String(rhs)) => lhs < rhs,
    (Value::String(lhs), Op::LessEq, Value::String(rhs)) => lhs <= rhs,
    (Value::String(lhs), Op::Greater, Value::String(rhs)) => lhs > rhs,
    (Value::String(lhs), Op::GreaterEq, Value::String(rhs)) => lhs >= rhs,
    // integer
    (Value::Int(lhs), Op::Eq, Value::Int(rhs)) => lhs == rhs,
    (Value::Int(lhs), Op::NotEq, Value::Int(rhs)) => lhs != rhs,
    (Value::Int(lhs), Op::Less, Value::Int(rhs)) => lhs < rhs,
    (Value::Int(lhs), Op::LessEq, Value::Int(rhs)) => lhs <= rhs,
    (Value::Int(lhs), Op::Greater, Value::Int(rhs)) => lhs > rhs,
    (Value::Int(lhs), Op::GreaterEq, Value::Int(rhs)) => lhs >= rhs,
    // float
    (Value::Float(lhs), Op::Eq, Value::Float(rhs)) => lhs == rhs,
    (Value::Float(lhs), Op::NotEq, Value::Float(rhs)) => lhs != rhs,
    (Value::Float(lhs), Op::Less, Value::Float(rhs)) => lhs < rhs,
    (Value::Float(lhs), Op::LessEq, Value::Float(rhs)) => lhs <= rhs,
    (Value::Float(lhs), Op::Greater, Value::Float(rhs)) => lhs > rhs,
    (Value::Float(lhs), Op::GreaterEq, Value::Float(rhs)) => lhs >= rhs,
    // boolean
    (Value::Boolean(lhs), Op::Eq, Value::Boolean(rhs)) => lhs == rhs,
    (Value::Boolean(lhs), Op::NotEq, Value::Boolean(rhs)) => lhs != rhs,
    // nil
    (Value::Nil, Op::Eq, Value::Nil) => true,
    (Value::Nil, Op::NotEq, Value::Nil) => false,
    (Value::Nil, Op::Eq, _) => false,
    (Value::Nil, Op::NotEq, _) => true,
    // mixed types and unsupported ops
    _ => false,
  }
}

#[derive(Debug, PartialEq)]
enum Value<'arena> {
  Int(isize),
  Float(f32),
  String(BumpString<'arena>),
  Boolean(bool),
  Nil,
}

#[derive(Debug, Copy, Clone)]
enum Op {
  Eq,
  NotEq,
  Less,
  LessEq,
  Greater,
  GreaterEq,
}

impl Op {
  fn from_str(op: &str) -> Option<Self> {
    match op {
      "==" => Some(Op::Eq),
      "!=" => Some(Op::NotEq),
      "<" => Some(Op::Less),
      "<=" => Some(Op::LessEq),
      ">" => Some(Op::Greater),
      ">=" => Some(Op::GreaterEq),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use test_utils::*;

  #[test]
  fn test_coerce() {
    let mut parser = test_parser!(adoc! {"
     :hello: world
     :bar: 2
     :defined_false!:
   "});
    parser.parse_document_header().unwrap();

    let cases = vec![
      ("true", Value::Boolean(true)),
      ("false", Value::Boolean(false)),
      ("", Value::Nil),
      ("1", Value::Int(1)),
      ("2", Value::Int(2)),
      ("3.23", Value::Float(3.23)),
      ("{not_defined}", Value::Nil),
      ("{defined_false}", Value::Nil),
      ("\"\"", Value::String(bstr!(""))),
      ("''", Value::String(bstr!(""))),
      ("    ", Value::String(bstr!(" "))),
      (".3", Value::Float(0.3)),
      ("not-enclosed-or-period", Value::Int(0)),
      ("'{hello}'", Value::String(bstr!("world"))),
      ("\"{hello}\"", Value::String(bstr!("world"))),
    ];
    for (input, expected) in cases {
      let actual = parser.coerce_eval_expr(input);
      assert_eq!(actual, expected);
    }
  }
}
