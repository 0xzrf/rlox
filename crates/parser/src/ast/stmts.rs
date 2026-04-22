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
    Expression {
        expr: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    Print {
        expr: Expr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    Block {
        stmts: Vec<Stmt>,
    },
    IfStmt {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
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
            Stmt::IfStmt { condition, then_branch, else_branch } => {
                if Interpret::is_truthy(&interpreter.evaluate(condition)?) {
                    then_branch.eval(interpreter)?;
                } else if let Some(else_branch) = else_branch {
                    else_branch.eval(interpreter)?;
                }

                Ok(())
            }
            Stmt::While { condition, body } => {
                while Interpret::is_truthy(&interpreter.evaluate(condition)?) {
                    body.eval(interpreter)?;
                }
                Ok(())
            }
        }
    }
}
