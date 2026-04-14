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

impl Expr {
    pub fn new_binary(left: Expr, operator: Token, right: Expr) -> Self {
        Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn new_unary(operator: Token, right: Self) -> Self {
        Expr::Unary { operator, right: Box::new(right) }
    }

    pub fn new_primary(value: Literal) -> Self {
        Expr::Literal { value }
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
