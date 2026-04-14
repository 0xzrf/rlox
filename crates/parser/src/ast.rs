use interpreter_types::Token;

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    Literal {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}

pub enum Literal {
    Number(String),
    String(String),
    True,
    False,
    Nil,
}

pub fn evaluate(expr: &Expr) {
    match expr {
        Expr::Binary { left, operator, right } => {
            // handle binary
            todo!()
        }
        Expr::Grouping { expression } => evaluate(expression),
        Expr::Literal { value } => todo!(),
        Expr::Unary { operator, right } => {
            // handle unary
            todo!()
        }
    }
}

// Presedence and associative rules for this context-free grammer
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;
