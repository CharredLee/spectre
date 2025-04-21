use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Float(f64),
    Function { params: Vec<String>, body: Term },
    Builtin(String),
    Unit,
}

#[derive(Clone)]
struct Environment {
    current: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
    syntax_rules: Vec<SyntaxRule>,
}

#[derive(Clone)]
struct SyntaxRule {
    name: String,
    pattern: String,
    precedence: usize,
    scope: Scope,
}

impl Environment {
    fn new() -> Self {
        let mut env = Environment {
            current: HashMap::new(),
            parent: None,
            syntax_rules: Vec::new(),
        };

        env.add_builtin("ID");

        env.add_syntax_rule(SyntaxRule {
            name: "FUNCTION".to_string(),
            pattern: "{name}({args})".to_string(),
            precedence: 1,
            scope: Scope::Global,
        });

        env
    }

    fn add_builtin(&mut self, name: &str) {
        self.current
            .insert(name.to_string(), Value::Builtin(name.to_string()));
    }

    fn add_syntax_rule(&mut self, rule: SyntaxRule) {
        self.syntax_rules.push(rule);
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        self.current
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.lookup(name)))
    }

    fn bind(&mut self, name: String, value: Value) {
        self.current.insert(name, value);
    }
}

pub struct Interpreter {
    env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: Environment::new(),
        }
    }

    pub fn interpret(&mut self, term: Term) -> Result<Value, String> {
        match term {
            Term::Identifier(name) => self
                .env
                .lookup(&name)
                .ok_or_else(|| format!("Undefined variable: {}", name)),
            Term::Integer(n) => Ok(Value::Integer(n)),
            Term::Float(f) => Ok(Value::Float(f)),
            Term::Function { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: *body,
                };
                self.env.bind(name, func.clone());
                Ok(func)
            }
            Term::FunctionCall { name, args } => match self.env.lookup(&name) {
                Some(Value::Builtin(builtin_name)) => self.call_builtin(&builtin_name, args),
                Some(Value::Function { params, body }) => {
                    if params.len() != args.len() {
                        return Err(format!(
                            "Arity mismatch: {} expected {} arguments, got {}",
                            name,
                            params.len(),
                            args.len()
                        ));
                    }

                    let mut new_env = Environment {
                        current: HashMap::new(),
                        parent: Some(Box::new(self.env.clone())),
                        syntax_rules: self.env.syntax_rules.clone(),
                    };

                    for (param, arg) in params.into_iter().zip(args) {
                        let value = self.interpret(arg)?;
                        new_env.bind(param, value);
                    }

                    let old_env = std::mem::replace(&mut self.env, new_env);
                    let result = self.interpret(body);
                    self.env = old_env;

                    result
                }
                Some(_) => Err(format!("{} is not a function", name)),
                None => Err(format!("Function not found: {}", name)),
            },
            Term::BinaryOp { op, left, right } => {
                let left_val = self.interpret(*left)?;
                let right_val = self.interpret(*right)?;
                self.apply_binary_op(op, left_val, right_val)
            }
            Term::UnaryOp { op, operand } => {
                let val = self.interpret(*operand)?;
                self.apply_unary_op(op, val)
            }
            Term::SyntaxDefinition {
                name,
                pattern,
                precedence,
                scope,
            } => {
                self.env.add_syntax_rule(SyntaxRule {
                    name,
                    pattern,
                    precedence,
                    scope,
                });
                Ok(Value::Unit)
            }
        }
    }

    fn call_builtin(&mut self, name: &str, args: Vec<Term>) -> Result<Value, String> {
        match name {
            "ID" => {
                if args.len() != 1 {
                    return Err("ID takes exactly one argument".to_string());
                }
                self.interpret(args.into_iter().next().unwrap())
            }
            _ => Err(format!("Unknown builtin: {}", name)),
        }
    }

    fn apply_binary_op(
        &self,
        op: BinaryOperator,
        left: Value,
        right: Value,
    ) -> Result<Value, String> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match op {
                BinaryOperator::Plus => Ok(Value::Integer(l + r)),
                BinaryOperator::Minus => Ok(Value::Integer(l - r)),
                BinaryOperator::Times => Ok(Value::Integer(l * r)),
                BinaryOperator::Div => {
                    if r == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Integer(l / r))
                    }
                }
                BinaryOperator::Pow => {
                    if r < 0 {
                        Ok(Value::Float((l as f64).powf(r as f64)))
                    } else if r > 20 {
                        Ok(Value::Float((l as f64).powf(r as f64)))
                    } else {
                        Ok(Value::Integer(l.pow(r as u32)))
                    }
                }
            },
            (Value::Float(l), Value::Float(r)) => match op {
                BinaryOperator::Plus => Ok(Value::Float(l + r)),
                BinaryOperator::Minus => Ok(Value::Float(l - r)),
                BinaryOperator::Times => Ok(Value::Float(l * r)),
                BinaryOperator::Div => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Value::Float(l / r))
                    }
                }
                BinaryOperator::Pow => Ok(Value::Float(l.powf(r))),
            },
            (Value::Integer(l), Value::Float(r)) => {
                self.apply_binary_op(op, Value::Float(l as f64), Value::Float(r))
            }
            (Value::Float(l), Value::Integer(r)) => {
                self.apply_binary_op(op, Value::Float(l), Value::Float(r as f64))
            }
            _ => Err("Invalid types for binary operation".to_string()),
        }
    }

    fn apply_unary_op(&self, op: UnaryOperator, val: Value) -> Result<Value, String> {
        match op {
            UnaryOperator::Neg => match val {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                Value::Float(f) => Ok(Value::Float(-f)),
                _ => Err("Invalid type for negation".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, Term, UnaryOperator};
    use std::f64::consts::PI;

    #[test]
    fn test_interpret_integer() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(Term::Integer(5));
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Integer(5) => {}
            _ => panic!("Expected integer 5"),
        }
    }

    #[test]
    fn test_interpret_float() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.interpret(Term::Float(PI));
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Float(f) if (f - PI).abs() < 0.001 => {}
            _ => panic!("Expected float PI"),
        }
    }

    #[test]
    fn test_interpret_binary_op_addition() {
        let mut interpreter = Interpreter::new();
        let ast = Term::BinaryOp {
            op: BinaryOperator::Plus,
            left: Box::new(Term::Integer(2)),
            right: Box::new(Term::Integer(3)),
        };
        let result = interpreter.interpret(ast);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Integer(5) => {}
            _ => panic!("Expected integer 5"),
        }
    }

    #[test]
    fn test_interpret_unary_op_negation() {
        let mut interpreter = Interpreter::new();
        let ast = Term::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Term::Integer(5)),
        };
        let result = interpreter.interpret(ast);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Integer(-5) => {}
            _ => panic!("Expected integer -5"),
        }
    }

    #[test]
    fn test_interpret_pow() {
        let mut interpreter = Interpreter::new();
        let ast = Term::BinaryOp {
            op: BinaryOperator::Pow,
            left: Box::new(Term::Integer(2)),
            right: Box::new(Term::Integer(3)),
        };
        let result = interpreter.interpret(ast);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Integer(8) => {}
            _ => panic!("Expected integer 8"),
        }
    }

    #[test]
    fn test_interpret_unary_minus_with_pow() {
        let mut interpreter = Interpreter::new();
        let ast = Term::UnaryOp {
            op: UnaryOperator::Neg,
            operand: Box::new(Term::BinaryOp {
                op: BinaryOperator::Pow,
                left: Box::new(Term::Integer(2)),
                right: Box::new(Term::Integer(2)),
            }),
        };
        let result = interpreter.interpret(ast);
        assert!(result.is_ok());
        match result.unwrap() {
            Value::Integer(-4) => {}
            _ => panic!("Expected integer -4"),
        }
    }
}
