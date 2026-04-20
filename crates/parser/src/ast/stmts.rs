// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;

use super::Expr;
use crate::Interpret;

#[derive(Debug)]
pub enum Stmt {
    Expression { expr: Expr },
    Print { expr: Expr },
}

impl Stmt {
    pub fn eval(&self) {
        match self {
            Stmt::Expression { expr } => {}
            Stmt::Print { expr } => {
                let value = Interpret::evaluate(expr).unwrap();
                println!("{value}");
            }
        }
    }
}
