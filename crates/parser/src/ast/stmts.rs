// program        → statement* EOF ;

// statement      → exprStmt
//                | printStmt ;

// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;


#[derive(Debug)]
pub enum Stmts {
    Expression,
    Print,
}
