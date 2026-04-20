// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

use interpreter_types::Token;

use super::Expr;
use crate::interpret::{Interpret, RuntimeError};

#[derive(Debug)]
pub enum Stmt {
    Expression { expr: Expr },
    Print { expr: Expr },
    Var { name: Token, initializer: Option<Expr> },
    Block { stmts: Vec<Stmt> },
}

impl Stmt {
    pub fn eval(&self, interpreter: &mut Interpret) -> Result<(), RuntimeError> {
        match self {
            Stmt::Expression { expr } => {
                interpreter.evaluate(expr)?;
                Ok(())
            }
            Stmt::Print { expr } => {
                let value = interpreter.evaluate(expr)?;
                println!("{value}");
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;
                if let Some(expr) = initializer {
                    value = Some(interpreter.evaluate(expr)?);
                }

                interpreter.env_define(name.lexeme.clone(), value);
                Ok(())
            }
            Stmt::Block { stmts } => {
                interpreter.execute_block(stmts)?;
                Ok(())
            }
        }
    }
}
