// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

use interpreter_types::Token;

use super::{Expr, Literal, LoxFunction};
use crate::interpret::{Interpret, RuntimeError, Value};

#[derive(Debug, Clone, PartialEq)]
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
    Return {
        keyword: Token,
        value: Option<Expr>,
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

pub enum StmtEvalType {
    None,
    Return(Value),
}

impl Stmt {
    pub fn eval(&self, interpreter: &mut Interpret) -> Result<StmtEvalType, RuntimeError> {
        match self {
            Stmt::Expression { expr } => {
                interpreter.evaluate(expr)?;
                Ok(StmtEvalType::None)
            }
            Stmt::Print { expr } => {
                let value = interpreter.evaluate(expr)?;
                println!("{value}");
                Ok(StmtEvalType::None)
            }
            Stmt::Var { name, initializer } => {
                let mut value = None;
                if let Some(expr) = initializer {
                    value = Some(interpreter.evaluate(expr)?);
                }

                interpreter.env_define(name.lexeme.clone(), value);
                Ok(StmtEvalType::None)
            }
            Stmt::Block { stmts } => {
                interpreter.execute_block(stmts)?;
                Ok(StmtEvalType::None)
            }
            Stmt::Return { keyword: _, value } => {
                let expr = value
                    .clone()
                    .and_then(|return_val| Some(interpreter.evaluate(&return_val)?))
                    .unwrap_or(Value::Nil);

                Ok(StmtEvalType::Return(expr))
            }
            Stmt::IfStmt { condition, then_branch, else_branch } => {
                if Interpret::is_truthy(&interpreter.evaluate(condition)?) {
                    then_branch.eval(interpreter)?;
                } else if let Some(else_branch) = else_branch {
                    else_branch.eval(interpreter)?;
                }

                Ok(StmtEvalType::None)
            }
            Stmt::While { condition, body } => {
                while Interpret::is_truthy(&interpreter.evaluate(condition)?) {
                    body.eval(interpreter)?;
                }
                Ok(StmtEvalType::None)
            }
            Stmt::Function { name, params, body } => {
                let function = Value::ForeignFn(LoxFunction::new(self.clone()));
                interpreter.env_define(name.lexeme.clone(), Some(function));
                Ok(StmtEvalType::None)
            }
        }
    }
}
