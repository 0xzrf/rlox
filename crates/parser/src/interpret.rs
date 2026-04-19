use std::fmt;

use interpreter_types::{Token, TokenType};

use crate::ast::{Expr, Literal};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => {
                let mut text = n.to_string();
                if text.ends_with(".0") {
                    text.truncate(text.len() - 2);
                }
                write!(f, "{text}")
            }
            Value::String(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Nil => write!(f, "nil"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub token: Token,
    pub message: String,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

pub type InterpretResult<T> = Result<T, RuntimeError>;

/// Tree-walk interpreter for expression ASTs.
pub struct Interpret;

impl Interpret {
    /// Evaluate an expression and return its runtime value.
    pub fn evaluate(expr: &Expr) -> InterpretResult<Value> {
        Self::eval(expr)
    }

    /// Convenience helper that evaluates and then stringifies the result,
    /// mirroring the behavior of jlox's `Interpreter.stringify()`.
    pub fn evaluate_to_string(expr: &Expr) -> InterpretResult<String> {
        Ok(Self::evaluate(expr)?.to_string())
    }

    fn eval(expr: &Expr) -> InterpretResult<Value> {
        use Expr::*;

        match expr {
            Literal { value } => Ok(Self::literal_to_value(value)),

            Grouping { expression } => Self::eval(expression),

            Unary { operator, right } => {
                let right_val = Self::eval(right)?;
                match operator.token_ty {
                    TokenType::MINUS => {
                        Self::check_number_operand(operator, &right_val)?;
                        if let Value::Number(n) = right_val {
                            Ok(Value::Number(-n))
                        } else {
                            unreachable!("check_number_operand should guarantee a number")
                        }
                    }
                    TokenType::BANG => Ok(Value::Bool(!Self::is_truthy(&right_val))),
                    _ => Ok(right_val),
                }
            }

            Binary { left, operator, right } => {
                let left_val = Self::eval(left)?;
                let right_val = Self::eval(right)?;

                use TokenType::*;

                match operator.token_ty {
                    MINUS => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l - r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    SLASH => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l / r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    STAR => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Number(l * r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    PLUS => match (left_val, right_val) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
                        (_, _) => Err(RuntimeError {
                            token: operator.clone(),
                            message: "Operands must be two numbers or two strings.".to_string(),
                        }),
                    },

                    GREATER => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Bool(l > r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    GREATER_EQUAL => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Bool(l >= r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    LESS => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Bool(l < r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }
                    LESS_EQUAL => {
                        Self::check_number_operands(operator, &left_val, &right_val)?;
                        if let (Value::Number(l), Value::Number(r)) = (left_val, right_val) {
                            Ok(Value::Bool(l <= r))
                        } else {
                            unreachable!("check_number_operands should guarantee numbers")
                        }
                    }

                    BANG_EQUAL => Ok(Value::Bool(!Self::is_equal(&left_val, &right_val))),
                    EQUAL_EQUAL => Ok(Value::Bool(Self::is_equal(&left_val, &right_val))),

                    _ => Ok(right_val),
                }
            }
        }
    }

    fn literal_to_value(lit: &Literal) -> Value {
        match lit {
            Literal::Number(s) => {
                let n = s.parse::<f64>().expect("scanner should only produce valid numbers");
                Value::Number(n)
            }
            Literal::String(s) => Value::String(s.clone()),
            Literal::True => Value::Bool(true),
            Literal::False => Value::Bool(false),
            Literal::Nil => Value::Nil,
        }
    }

    fn is_truthy(value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(a: &Value, b: &Value) -> bool {
        use Value::*;
        match (a, b) {
            (Nil, Nil) => true,
            (Nil, _) | (_, Nil) => false,
            (Bool(x), Bool(y)) => x == y,
            (Number(x), Number(y)) => x == y,
            (String(x), String(y)) => x == y,
            _ => false,
        }
    }

    fn check_number_operand(operator: &Token, operand: &Value) -> InterpretResult<()> {
        if matches!(operand, Value::Number(_)) {
            Ok(())
        } else {
            Err(RuntimeError {
                token: operator.clone(),
                message: "Operand must be a number.".to_string(),
            })
        }
    }

    fn check_number_operands(operator: &Token, left: &Value, right: &Value) -> InterpretResult<()> {
        if matches!((left, right), (Value::Number(_), Value::Number(_))) {
            Ok(())
        } else {
            Err(RuntimeError {
                token: operator.clone(),
                message: "Operands must be numbers.".to_string(),
            })
        }
    }
}


#[cfg(test)]
mod interpret_tests {
    use scanner::Scanner;

    use super::{Interpret, RuntimeError, Value};
    use crate::{Expr, Parser, ParserResult};
    fn get_parse_result(source_code: &str) -> ParserResult<Expr> {
        let tokens = Scanner::_new(source_code.to_string()).scan(false).unwrap().0.get_tokens();

        Parser::new(&tokens).parse()
    }

    fn get_eval(source_code: &str) -> Result<Value, RuntimeError> {
        Interpret::eval(
            &get_parse_result(source_code).expect("Couldn't parse the value the value properly"),
        )
    }

    #[test]
    pub fn test_eval() {
        let source_code = "2 + 3";

        let eval = get_eval(source_code);
        println!("eval: {eval:#?}");

        let Ok(Value::Number(val)) = eval else { panic!() };
        assert_eq!(val, 5.0);
    }

    #[test]
    pub fn test_err_when_wrong_expr() {
        let source_code = "2 * (3 / -\"muffin\")";

        let eval = get_eval(source_code);

        assert!(eval.is_err(), "expected this to fail");
    }

    #[test]
    pub fn test_bool() {
        let eval = get_eval("true");

        assert!(eval.is_ok(), "Expected the evaluation to pass");
        assert_eq!(eval.unwrap(), Value::Bool(true), "Unexpected eval value");
    }

    #[test]
    fn string_concatenation_with_plus() {
        assert_eq!(
            get_eval("\"hello\" + \" \" + \"world\"").unwrap(),
            Value::String("hello world".to_string())
        );
    }

    #[test]
    fn unary_minus_negates_number() {
        assert_eq!(get_eval("- (1 + 2)").unwrap(), Value::Number(-3.0));
    }

    #[test]
    fn comparison_operators_produce_bool() {
        assert_eq!(get_eval("3 < 4").unwrap(), Value::Bool(true));
        assert_eq!(get_eval("5 <= 5").unwrap(), Value::Bool(true));
        assert_eq!(get_eval("10 > 3").unwrap(), Value::Bool(true));
    }

    #[test]
    fn equality_on_booleans_numbers_and_strings() {
        assert_eq!(get_eval("true == false").unwrap(), Value::Bool(false));
        assert_eq!(get_eval("\"a\" == \"a\"").unwrap(), Value::Bool(true));
        assert_eq!(get_eval("1 == 2").unwrap(), Value::Bool(false));
        assert_eq!(get_eval("1 != 2").unwrap(), Value::Bool(true));
    }

    #[test]
    fn bang_truthiness_like_lox() {
        assert_eq!(get_eval("!false").unwrap(), Value::Bool(true));
        assert_eq!(get_eval("!true").unwrap(), Value::Bool(false));
        assert_eq!(get_eval("!0").unwrap(), Value::Bool(false));
    }

    #[test]
    fn grouping_changes_precedence() {
        assert_eq!(get_eval("(1 + 2) * 3").unwrap(), Value::Number(9.0));
    }

    #[test]
    fn plus_runtime_error_when_operand_types_mismatch() {
        let err = get_eval("1 + \"a\"").expect_err("number + string should error");
        assert!(
            err.message.contains("Operands must be two numbers or two strings"),
            "unexpected message: {}",
            err.message
        );
    }
}
