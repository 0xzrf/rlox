// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

use interpreter_types::Token;

use super::Expr;
use crate::Interpret;

#[derive(Debug)]
pub enum Stmt {
    ExpressionStmt { expr: Expr },
    Print { expr: Expr },
    Var { name: Token, initilizer: Option<Expr> },
}

impl Stmt {
    pub fn eval(&self) {
        match self {
            Stmt::ExpressionStmt { expr } => {}
            Stmt::Print { expr } => {
                let value = Interpret::evaluate(expr).unwrap();
                println!("{value}");
            }
            Stmt::Var { name, initilizer } => {}
        }
    }
}
