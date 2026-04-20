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

use interpreter_types::Token;

#[derive(Debug)]
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
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: Box<Expr>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(String),
    String(String),
    True,
    False,
    Nil,
}

use std::fmt;

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::Number(val) => write!(f, "{val}"),
            Literal::String(val) => write!(f, "\"{val}\""),
            Literal::True => write!(f, "true"),
            Literal::False => write!(f, "false"),
            Literal::Nil => write!(f, "nil"),
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

    pub fn new_grouping(expr: Expr) -> Self {
        Expr::Grouping { expression: Box::new(expr) }
    }

    pub fn new_variable(name: Token) -> Self {
        Expr::Variable { name }
    }

    pub fn get_stringified_expr(&self) -> String {
        let parenthesize = AstPrinter::parenthesize;
        match self {
            Expr::Binary { left, operator, right } => {
                parenthesize(&operator.lexeme, &[left, right])
            }
            Expr::Grouping { expression } => parenthesize("group", &[expression]),
            Expr::Literal { value } => {
                if *value == Literal::Nil {
                    return "nil".to_string();
                }
                format!("{value}")
            }
            Expr::Unary { operator, right } => parenthesize(&operator.lexeme, &[right]),
            Expr::Variable { name } => name.lexeme.clone(),
            Expr::Assign { name, value } => parenthesize(&format!("= {}", name.lexeme), &[value]),
        }
    }
}

pub struct AstPrinter;

impl AstPrinter {
    pub fn print(expr: &Expr) -> String {
        expr.get_stringified_expr()
    }

    fn parenthesize(name: &str, exprs: &[&Expr]) -> String {
        let mut str_buf = String::new();

        str_buf.push('(');
        str_buf.push_str(name);
        for expr in exprs {
            str_buf.push(' ');
            str_buf.push_str(&expr.get_stringified_expr());
        }
        str_buf.push(')');

        str_buf
    }
}
