// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

use interpreter_types::Token;

use super::Expr;
use crate::Interpret;
use crate::env::Env;
use crate::interpret::RuntimeError;

#[derive(Debug)]
pub enum Stmt {
    ExpressionStmt { expr: Expr },
    Print { expr: Expr },
    Var { name: Token, initializer: Option<Expr> },
}

impl Stmt {
    pub fn eval(&self, env: &mut Env) -> Result<(), RuntimeError> {
        match self {
            Stmt::ExpressionStmt { expr } => Ok(()),
            Stmt::Print { expr } => {
                let value = Interpret::evaluate(expr).unwrap();
                println!("{value}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;
                if let Some(expr) = initializer {
                    value = Some(Interpret::evaluate(expr)?);
                }

                env.define(name.lexeme, value);
                Ok(())
            }
        }
    }
}
