use parser::{Expr, Literal, Parser, Stmt};
use scanner::Scanner;

fn parse_program(source_code: &str) -> Vec<Stmt> {
    let tokens = Scanner::_new(source_code.to_string()).scan(false).unwrap().0.get_tokens();
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

#[test]
fn parses_var_declaration_without_initializer() {
    let mut stmts = parse_program("var a;");
    assert_eq!(stmts.len(), 1);

    let Stmt::Var { name, initializer } = stmts.remove(0) else {
        panic!("expected var declaration stmt");
    };

    assert_eq!(name.lexeme, "a");
    assert!(initializer.is_none(), "expected no initializer");
}

#[test]
fn parses_block_statement_with_multiple_statements() {
    let mut stmts = parse_program("{ var a = 1; print a; }");
    assert_eq!(stmts.len(), 1);

    let Stmt::Block { stmts: inner } = stmts.remove(0) else {
        panic!("expected block stmt");
    };
    assert_eq!(inner.len(), 2);

    let Stmt::Var { name, initializer } = &inner[0] else {
        panic!("expected var stmt as first block stmt");
    };
    assert_eq!(name.lexeme, "a");
    let Some(Expr::Literal { value }) = initializer else {
        panic!("expected initializer expr");
    };
    assert_eq!(value, &Literal::Number("1.0".to_string()));

    let Stmt::Print { expr } = &inner[1] else {
        panic!("expected print stmt as second block stmt");
    };
    let Expr::Variable { name } = expr else {
        panic!("expected variable expr in print");
    };
    assert_eq!(name.lexeme, "a");
}

#[test]
fn parses_assignment_expression_statement() {
    let mut stmts = parse_program("a = 3;");
    assert_eq!(stmts.len(), 1);

    let Stmt::Expression { expr } = stmts.remove(0) else {
        panic!("expected expression stmt");
    };

    let Expr::Assign { name, value } = expr else {
        panic!("expected assign expr");
    };
    assert_eq!(name.lexeme, "a");
    let Expr::Literal { value } = *value else {
        panic!("expected literal right-hand side");
    };
    assert_eq!(value, Literal::Number("3.0".to_string()));
}

#[test]
fn parses_multiple_top_level_statements_in_order() {
    let stmts = parse_program("var a = 1; print a;");
    assert_eq!(stmts.len(), 2);

    let Stmt::Var { name, initializer } = &stmts[0] else {
        panic!("expected var stmt first");
    };
    assert_eq!(name.lexeme, "a");
    let Some(Expr::Literal { value }) = initializer else {
        panic!("expected initializer literal");
    };
    assert_eq!(value, &Literal::Number("1.0".to_string()));

    let Stmt::Print { expr } = &stmts[1] else {
        panic!("expected print stmt second");
    };
    let Expr::Variable { name } = expr else {
        panic!("expected variable expr in print");
    };
    assert_eq!(name.lexeme, "a");
}

#[test]
fn parses_print_statement_with_string_literal() {
    let mut stmts = parse_program("print \"hello\";");
    assert_eq!(stmts.len(), 1);

    let Stmt::Print { expr } = stmts.remove(0) else {
        panic!("expected print stmt");
    };

    let Expr::Literal { value } = expr else {
        panic!("expected literal expr");
    };
    assert_eq!(value, Literal::String("hello".to_string()));
}

#[test]
fn parses_nested_block_statements() {
    let mut stmts = parse_program("{ { var a = 1; } }");
    assert_eq!(stmts.len(), 1);

    let Stmt::Block { stmts: outer } = stmts.remove(0) else {
        panic!("expected outer block");
    };
    assert_eq!(outer.len(), 1);

    let Stmt::Block { stmts: inner } = &outer[0] else {
        panic!("expected inner block");
    };
    assert_eq!(inner.len(), 1);

    let Stmt::Var { name, initializer } = &inner[0] else {
        panic!("expected var stmt inside inner block");
    };
    assert_eq!(name.lexeme, "a");
    let Some(Expr::Literal { value }) = initializer else {
        panic!("expected initializer");
    };
    assert_eq!(value, &Literal::Number("1.0".to_string()));
}

#[test]
fn parses_empty_block_statement() {
    let mut stmts = parse_program("{}");
    assert_eq!(stmts.len(), 1);

    let Stmt::Block { stmts: inner } = stmts.remove(0) else {
        panic!("expected block stmt");
    };
    assert!(inner.is_empty(), "expected empty block");
}

#[test]
fn parses_var_initializer_as_binary_expression() {
    let mut stmts = parse_program("var a = 1 + 2;");
    assert_eq!(stmts.len(), 1);

    let Stmt::Var { name, initializer } = stmts.remove(0) else {
        panic!("expected var stmt");
    };
    assert_eq!(name.lexeme, "a");

    let Some(Expr::Binary { left, operator, right }) = initializer else {
        panic!("expected binary initializer");
    };
    assert_eq!(operator.lexeme, "+");

    let Expr::Literal { value: left_val } = *left else {
        panic!("expected literal left operand");
    };
    let Expr::Literal { value: right_val } = *right else {
        panic!("expected literal right operand");
    };

    assert_eq!(left_val, Literal::Number("1.0".to_string()));
    assert_eq!(right_val, Literal::Number("2.0".to_string()));
}
