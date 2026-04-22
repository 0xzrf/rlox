use parser::interpret::Value;
use parser::{Expr, Interpret, Literal, Parser, Stmt};
use scanner::Scanner;

fn parse_program(source_code: &str) -> Vec<Stmt> {
    let tokens = Scanner::_new(source_code.to_string()).scan(false).unwrap().0.get_tokens();
    Parser::new(&tokens).parse().unwrap()
}

fn interpret_program(source_code: &str) -> Interpret {
    let stmts = parse_program(source_code);
    let mut interpreter = Interpret::new();
    interpreter.interpret_stmts(&stmts).unwrap();
    interpreter
}

#[test]
fn parses_function_declaration_with_no_params_and_empty_body() {
    let mut stmts = parse_program("fun f() {}");
    assert_eq!(stmts.len(), 1);

    let Stmt::Function { name, params, body } = stmts.remove(0) else {
        panic!("expected function declaration stmt");
    };

    assert_eq!(name.lexeme, "f");
    assert!(params.is_empty(), "expected no params");
    assert!(body.is_empty(), "expected empty body");
}

#[test]
fn parses_function_declaration_with_params_and_body_statements() {
    let mut stmts = parse_program("fun add(a, b) { print a; return b; }");
    assert_eq!(stmts.len(), 1);

    let Stmt::Function { name, params, body } = stmts.remove(0) else {
        panic!("expected function declaration stmt");
    };

    assert_eq!(name.lexeme, "add");
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].lexeme, "a");
    assert_eq!(params[1].lexeme, "b");

    assert_eq!(body.len(), 2);
    assert!(matches!(body[0], Stmt::Print { .. }), "expected print stmt in body");
    assert!(matches!(body[1], Stmt::Return { .. }), "expected return stmt in body");
}

#[test]
fn parses_multiple_function_declarations_in_order() {
    let stmts = parse_program("fun a() {} fun b(x) { return x; }");
    assert_eq!(stmts.len(), 2);

    let Stmt::Function { name, params, body } = &stmts[0] else {
        panic!("expected first stmt to be a function declaration");
    };
    assert_eq!(name.lexeme, "a");
    assert!(params.is_empty());
    assert!(body.is_empty());

    let Stmt::Function { name, params, body } = &stmts[1] else {
        panic!("expected second stmt to be a function declaration");
    };
    assert_eq!(name.lexeme, "b");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].lexeme, "x");
    assert_eq!(body.len(), 1);
    assert!(matches!(body[0], Stmt::Return { .. }));
}

#[test]
fn parses_function_body_with_nested_block_statement() {
    let mut stmts = parse_program("fun f() { { print 1; } }");
    assert_eq!(stmts.len(), 1);

    let Stmt::Function { name, params, body } = stmts.remove(0) else {
        panic!("expected function declaration stmt");
    };
    assert_eq!(name.lexeme, "f");
    assert!(params.is_empty());
    assert_eq!(body.len(), 1);

    let Stmt::Block { stmts: inner } = &body[0] else {
        panic!("expected nested block stmt in function body");
    };
    assert_eq!(inner.len(), 1);
    assert!(matches!(inner[0], Stmt::Print { .. }));
}

#[test]
fn errors_on_function_declaration_missing_name() {
    let tokens = Scanner::_new("fun () {}".to_string()).scan(false).unwrap().0.get_tokens();
    let err = Parser::new(&tokens).parse().unwrap_err();
    assert!(
        err.to_string().contains("Expected function name"),
        "unexpected error message: {err}"
    );
}

#[test]
fn errors_on_function_declaration_missing_right_paren_after_params() {
    let src = "fun f(a, b { return a; }";
    let tokens = Scanner::_new(src.to_string()).scan(false).unwrap().0.get_tokens();
    let err = Parser::new(&tokens).parse().unwrap_err();
    assert!(
        err.to_string().contains("Expected ) during function declaration"),
        "unexpected error message: {err}"
    );
}

#[test]
fn parses_function_declaration_with_255_parameters() {
    let params = (0..255)
        .map(|i| format!("p{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let src = format!("fun many({params}) {{}}");

    let mut stmts = parse_program(&src);
    assert_eq!(stmts.len(), 1);

    let Stmt::Function { name, params, body } = stmts.remove(0) else {
        panic!("expected function declaration stmt");
    };

    assert_eq!(name.lexeme, "many");
    assert_eq!(params.len(), 255);
    assert!(body.is_empty());
}

#[test]
fn parses_function_body_with_for_loop_statement() {
    let mut stmts = parse_program("fun f(n) { for (var i = 0; i < n; i = i + 1) print i; }");
    assert_eq!(stmts.len(), 1);

    let Stmt::Function { name, params, body } = stmts.remove(0) else {
        panic!("expected function declaration stmt");
    };

    assert_eq!(name.lexeme, "f");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].lexeme, "n");
    assert_eq!(body.len(), 1);

    // `for` desugars into a while/block structure.
    assert!(
        matches!(body[0], Stmt::Block { .. } | Stmt::While { .. }),
        "expected desugared for loop statement in body, got: {:?}",
        body[0]
    );
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

#[test]
fn parses_print_statement_with_binary_expression() {
    let mut stmts = parse_program("print 1 + 2;");
    assert_eq!(stmts.len(), 1);

    let Stmt::Print { expr } = stmts.remove(0) else {
        panic!("expected print stmt");
    };

    let Expr::Binary { left, operator, right } = expr else {
        panic!("expected binary expr");
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

#[test]
fn parses_block_with_trailing_whitespace_and_newlines() {
    let mut stmts = parse_program("{\n  var a = 1;\n  print a;\n}\n");
    assert_eq!(stmts.len(), 1);

    let Stmt::Block { stmts: inner } = stmts.remove(0) else {
        panic!("expected block stmt");
    };
    assert_eq!(inner.len(), 2);
}

#[test]
fn parses_var_initializer_grouping_expression() {
    let mut stmts = parse_program("var a = (1);");
    assert_eq!(stmts.len(), 1);

    let Stmt::Var { name, initializer } = stmts.remove(0) else {
        panic!("expected var stmt");
    };
    assert_eq!(name.lexeme, "a");

    let Some(Expr::Grouping { expression }) = initializer else {
        panic!("expected grouping initializer");
    };
    let Expr::Literal { value } = *expression else {
        panic!("expected literal inside grouping");
    };
    assert_eq!(value, Literal::Number("1.0".to_string()));
}

#[test]
fn parses_print_statement_grouped_variable() {
    let mut stmts = parse_program("print (a);");
    assert_eq!(stmts.len(), 1);

    let Stmt::Print { expr } = stmts.remove(0) else {
        panic!("expected print stmt");
    };
    let Expr::Grouping { expression } = expr else {
        panic!("expected grouping expr");
    };
    let Expr::Variable { name } = *expression else {
        panic!("expected variable inside grouping");
    };
    assert_eq!(name.lexeme, "a");
}

#[test]
fn parses_while_statement_with_block_body() {
    let mut stmts = parse_program("while (a < 10) { a = a + 1; }");
    assert_eq!(stmts.len(), 1);

    let Stmt::While { condition, body } = stmts.remove(0) else {
        panic!("expected while stmt");
    };

    let Expr::Binary { left, operator, right } = condition else {
        panic!("expected binary condition");
    };
    assert_eq!(operator.lexeme, "<");

    let Expr::Variable { name: left_name } = *left else {
        panic!("expected variable on left side of condition");
    };
    assert_eq!(left_name.lexeme, "a");

    let Expr::Literal { value: right_value } = *right else {
        panic!("expected literal on right side of condition");
    };
    assert_eq!(right_value, Literal::Number("10.0".to_string()));

    let Stmt::Block { stmts: body_stmts } = *body else {
        panic!("expected block body");
    };
    assert_eq!(body_stmts.len(), 1);
}

#[test]
fn while_statement_executes_until_condition_is_false() {
    let mut interpreter = interpret_program(
        r#"
        var i = 0;
        while (i < 3) {
          i = i + 1;
        }
        "#,
    );

    let i_value = interpreter
        .evaluate(&Expr::Variable {
            name: interpreter_types::Token::new(
                interpreter_types::TokenType::IDENTIFIER,
                1,
                "i".to_string(),
                0,
                String::new(),
            ),
        })
        .unwrap();

    assert_eq!(i_value, Value::Number(3.0));
}
