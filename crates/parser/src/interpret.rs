use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

use interpreter_types::{Token, TokenType};

use crate::ast::{Expr, Literal, Stmt};
use crate::env::{Env, EnvRef};

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
pub struct Interpret {
    env: EnvRef,
}

impl Interpret {
    pub fn new() -> Self {
        Self {
            env: Rc::new(RefCell::new(Env::new(None))),
        }
    }

    pub fn interpret_stmts(&mut self, stmts: &[Stmt]) -> InterpretResult<()> {
        for stmt in stmts {
            stmt.eval(self)?;
        }
        Ok(())
    }

    /// Evaluate an expression and return its runtime value.
    pub fn evaluate(&mut self, expr: &Expr) -> InterpretResult<Value> {
        self.eval(expr)
    }

    /// Convenience helper that evaluates and then stringifies the result,
    /// mirroring the behavior of jlox's `Interpreter.stringify()`.
    pub fn evaluate_to_string(&mut self, expr: &Expr) -> InterpretResult<String> {
        Ok(self.evaluate(expr)?.to_string())
    }

    fn eval(&mut self, expr: &Expr) -> InterpretResult<Value> {
        use Expr::*;

        match expr {
            Literal { value } => Ok(Self::literal_to_value(value)),

            Grouping { expression } => self.eval(expression),

            Unary { operator, right } => {
                let right_val = self.eval(right)?;
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

            Variable { name } => self.get_variable(name),

            Assign { name, value } => {
                let eval = self.eval(value)?;
                self.env
                    .borrow_mut()
                    .assign(name.lexeme.clone(), eval.clone())
                    .map_err(|msg| RuntimeError {
                        token: name.clone(),
                        message: msg,
                    })?;
                return Ok(eval);
            }

            Binary { left, operator, right } => {
                let left_val = self.eval(left)?;
                let right_val = self.eval(right)?;

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

impl Interpret {
    fn get_variable(&mut self, name: &Token) -> InterpretResult<Value> {
        let Some(val) = self.env.borrow().get_owned(&name.lexeme) else {
            return Err(RuntimeError {
                token: name.clone(),
                message: format!("Undefined variable '{}'.", name.lexeme),
            });
        };
        Ok(val)
    }

    pub(crate) fn env_define(&mut self, name: String, value: Option<Value>) {
        self.env.borrow_mut().define(name, value);
    }

    pub(crate) fn execute_block(&mut self, stmts: &[Stmt]) -> InterpretResult<()> {
        let previous = self.env.clone();
        self.env = Rc::new(RefCell::new(Env::new(Some(previous.clone()))));

        let result = (|| {
            for stmt in stmts {
                stmt.eval(self)?;
            }
            Ok(())
        })();

        self.env = previous;
        result
    }

    pub(crate) fn with_env(&mut self, env: EnvRef) -> EnvRef {
        let prev = self.env.clone();
        self.env = env;
        prev
    }
}


// #[cfg(test)]
// mod interpret_tests {
//     use scanner::Scanner;

//     use super::{Interpret, RuntimeError, Value};
//     use crate::{Expr, Parser, ParserResult, Stmt};

//     fn get_parse_result(source_code: &str) -> ParserResult<Expr> {
//         let mut src = source_code.trim().to_string();
//         if !src.ends_with(';') {
//             src.push(';');
//         }

//         let tokens = Scanner::_new(src).scan(false).unwrap().0.get_tokens();
//         let mut stmts = Parser::new(&tokens).parse()?;

//         assert_eq!(stmts.len(), 1, "interpret tests expect a single expression statement");

//         match stmts.remove(0) {
//             Stmt::Expression { expr } => Ok(expr),
//             Stmt::Print { .. } => {
//                 panic!("interpret tests expect an expression statement, got print")
//             }
//         }
//     }

//     fn get_eval(source_code: &str) -> Result<Value, RuntimeError> {
//         Interpret::eval(
//             &get_parse_result(source_code).expect("Couldn't parse the value the value properly"),
//         )
//     }

//     #[test]
//     pub fn test_eval() {
//         let source_code = "2 + 3";

//         let eval = get_eval(source_code);
//         println!("eval: {eval:#?}");

//         let Ok(Value::Number(val)) = eval else { panic!() };
//         assert_eq!(val, 5.0);
//     }

//     #[test]
//     pub fn test_err_when_wrong_expr() {
//         let source_code = "2 * (3 / -\"muffin\")";

//         let eval = get_eval(source_code);

//         assert!(eval.is_err(), "expected this to fail");
//     }

//     #[test]
//     pub fn test_bool() {
//         let eval = get_eval("true");

//         assert!(eval.is_ok(), "Expected the evaluation to pass");
//         assert_eq!(eval.unwrap(), Value::Bool(true), "Unexpected eval value");
//     }

//     #[test]
//     fn string_concatenation_with_plus() {
//         assert_eq!(
//             get_eval("\"hello\" + \" \" + \"world\"").unwrap(),
//             Value::String("hello world".to_string())
//         );
//     }

//     #[test]
//     fn unary_minus_negates_number() {
//         assert_eq!(get_eval("- (1 + 2)").unwrap(), Value::Number(-3.0));
//     }

//     #[test]
//     fn comparison_operators_produce_bool() {
//         assert_eq!(get_eval("3 < 4").unwrap(), Value::Bool(true));
//         assert_eq!(get_eval("5 <= 5").unwrap(), Value::Bool(true));
//         assert_eq!(get_eval("10 > 3").unwrap(), Value::Bool(true));
//     }

//     #[test]
//     fn equality_on_booleans_numbers_and_strings() {
//         assert_eq!(get_eval("true == false").unwrap(), Value::Bool(false));
//         assert_eq!(get_eval("\"a\" == \"a\"").unwrap(), Value::Bool(true));
//         assert_eq!(get_eval("1 == 2").unwrap(), Value::Bool(false));
//         assert_eq!(get_eval("1 != 2").unwrap(), Value::Bool(true));
//     }

//     #[test]
//     fn bang_truthiness_like_lox() {
//         assert_eq!(get_eval("!false").unwrap(), Value::Bool(true));
//         assert_eq!(get_eval("!true").unwrap(), Value::Bool(false));
//         assert_eq!(get_eval("!0").unwrap(), Value::Bool(false));
//     }

//     #[test]
//     fn grouping_changes_precedence() {
//         assert_eq!(get_eval("(1 + 2) * 3").unwrap(), Value::Number(9.0));
//     }

//     #[test]
//     fn plus_runtime_error_when_operand_types_mismatch() {
//         let err = get_eval("1 + \"a\"").expect_err("number + string should error");
//         assert!(
//             err.message.contains("Operands must be two numbers or two strings"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn division_chains_left_to_right() {
//         assert_eq!(get_eval("24 / 3 / 2").unwrap(), Value::Number(4.0));
//     }

//     #[test]
//     fn unary_minus_errors_on_non_number_operand() {
//         let err = get_eval("-true").expect_err("- on bool should error");
//         assert!(
//             err.message.contains("Operand must be a number"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn multiplication_binds_tighter_than_addition() {
//         assert_eq!(get_eval("1 + 2 * 3").unwrap(), Value::Number(7.0));
//     }

//     #[test]
//     fn comparison_errors_when_operands_are_not_both_numbers() {
//         let err = get_eval("3 < true").expect_err("number vs bool comparison should error");
//         assert!(
//             err.message.contains("Operands must be numbers"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn equality_works_across_mixed_operand_types() {
//         assert_eq!(get_eval("1 == true").unwrap(), Value::Bool(false));
//         assert_eq!(get_eval("\"hi\" != 1").unwrap(), Value::Bool(true));
//     }

//     #[test]
//     fn chained_equality_is_left_associative() {
//         assert_eq!(get_eval("true == false == false").unwrap(), Value::Bool(true));
//         assert_eq!(get_eval("1 == 2 != 3").unwrap(), Value::Bool(true));
//     }

//     #[test]
//     fn evaluate_to_string_formats_like_lox_numbers() {
//         let expr = get_parse_result("42").expect("parse");
//         assert_eq!(Interpret::evaluate_to_string(&expr).unwrap(), "42");
//     }

//     #[test]
//     fn subtraction_chains_left_to_right() {
//         assert_eq!(get_eval("10 - 3 - 2").unwrap(), Value::Number(5.0));
//     }

//     #[test]
//     fn addition_and_multiplication_mix_respects_precedence() {
//         assert_eq!(get_eval("1 + 2 * 3 + 4").unwrap(), Value::Number(11.0));
//     }

//     #[test]
//     fn unary_bang_binds_tighter_than_equality() {
//         assert_eq!(get_eval("!false == true").unwrap(), Value::Bool(true));
//     }

//     #[test]
//     fn star_errors_when_operands_are_not_numbers() {
//         let err = get_eval("3 * true").expect_err("* with bool should error");
//         assert!(
//             err.message.contains("Operands must be numbers"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn slash_errors_when_operands_are_not_numbers() {
//         let err = get_eval("\"x\" / 2").expect_err("string / number should error");
//         assert!(
//             err.message.contains("Operands must be numbers"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn nested_grouping_preserves_literal_value() {
//         assert_eq!(get_eval("((42))").unwrap(), Value::Number(42.0));
//     }

//     #[test]
//     fn grouped_subexpression_precedence_inside_factors() {
//         assert_eq!(get_eval("(1 + 2) * (3 - 4 / 2)").unwrap(), Value::Number(3.0));
//     }

//     #[test]
//     fn minus_errors_when_operand_is_not_a_number() {
//         let err = get_eval("\"a\" - 1").expect_err("string - number should error");
//         assert!(
//             err.message.contains("Operands must be numbers"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn chained_comparison_errors_when_result_is_not_comparable_as_number() {
//         let err = get_eval("3 > 2 > 1").expect_err("bool compared to number should error");
//         assert!(
//             err.message.contains("Operands must be numbers"),
//             "unexpected message: {}",
//             err.message
//         );
//     }

//     #[test]
//     fn chained_unary_minus_double_negates() {
//         assert_eq!(get_eval("--1").unwrap(), Value::Number(1.0));
//     }

//     #[test]
//     fn bang_on_string_uses_truthiness() {
//         assert_eq!(get_eval("!\"hi\"").unwrap(), Value::Bool(false));
//     }

//     #[test]
//     fn evaluate_to_string_formats_boolean_literals() {
//         let expr = get_parse_result("false").expect("parse");
//         assert_eq!(Interpret::evaluate_to_string(&expr).unwrap(), "false");
//     }

//     #[test]
//     fn string_inequality_compares_lexeme_payloads() {
//         assert_eq!(get_eval("\"a\" != \"b\"").unwrap(), Value::Bool(true));
//     }
// }
