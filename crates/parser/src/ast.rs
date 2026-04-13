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
        value: String,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
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
