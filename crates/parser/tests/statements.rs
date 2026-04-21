use parser::{Expr, Literal, Parser, Stmt};
use scanner::Scanner;

fn parse_program(source_code: &str) -> Vec<Stmt> {
    let tokens = Scanner::_new(source_code.to_string())
        .scan(false)
        .unwrap()
        .0
        .get_tokens();
    Parser::new(&tokens).parse().unwrap()
}

#[test]
fn parses_var_declaration_with_initializer() {
    let mut stmts = parse_program("var a = \"hi\";");
    assert_eq!(stmts.len(), 1);

    let Stmt::Var { name, initializer } = stmts.remove(0) else {
        panic!("expected var declaration stmt");
    };

    assert_eq!(name.lexeme, "a");
    let Some(Expr::Literal { value }) = initializer else {
        panic!("expected literal initializer");
    };
    assert_eq!(value, Literal::String("hi".to_string()));
}

#[test]
fn parses_print_statement_with_variable_expression() {
    let mut stmts = parse_program("print a;");
    assert_eq!(stmts.len(), 1);

    let Stmt::Print { expr } = stmts.remove(0) else {
        panic!("expected print stmt");
    };

    let Expr::Variable { name } = expr else {
        panic!("expected variable expr");
    };
    assert_eq!(name.lexeme, "a");
}

