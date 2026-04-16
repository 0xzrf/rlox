#[cfg(test)]
pub mod parser_tests {
    use parser::{AstPrinter, Expr, Parser, ParserResult};
    use scanner::Scanner;

    fn get_parse_result(source_code: &str) -> ParserResult<Expr> {
        let tokens = Scanner::_new(source_code.to_string()).scan(false).unwrap().0.get_tokens();

        Parser::new(&tokens).parse()
    }

    fn parse_code_and_return_ast(source_code: &str) -> Option<String> {
        let parsed_expr = get_parse_result(source_code).ok()?;

        Some(AstPrinter::print(&parsed_expr))
    }

    #[test]
    // #[ignore = "internal test function"]
    fn test_syntax_tree() {
        let source_code = "../../test.lox";

        let tokens = Scanner::new(source_code.to_string()).scan(false).unwrap().0.get_tokens();

        let parser_result = Parser::new(&tokens).parse();

        assert!(
            parser_result.is_ok(),
            "expected parser to not fail here, got: {}",
            parser_result.unwrap_err()
        );

        println!("{:#?}", AstPrinter::print(&parser_result.unwrap()));
    }

    #[test]
    fn test_parses_binary() {
        let source_code = "2 + 3";

        let parse_result = parse_code_and_return_ast(source_code);

        assert!(parse_result.is_some(), "expected the parser to output a value for the lox code");
        assert_eq!(parse_result.unwrap(), "(+ 2.0 3.0)", "The parsed value doesn't match");
    }

    #[test]
    fn test_operator_precedence_multiplication_over_addition() {
        let source_code = "1 + 2 * 3";
        let parse_result = parse_code_and_return_ast(source_code).unwrap();
        assert_eq!(parse_result, "(+ 1.0 (* 2.0 3.0))");
    }

    #[test]
    fn test_grouping_overrides_precedence() {
        let source_code = "(1 + 2) * 3";
        let parse_result = parse_code_and_return_ast(source_code).unwrap();
        assert_eq!(parse_result, "(* (group (+ 1.0 2.0)) 3.0)");
    }

    #[test]
    fn test_unary_minus() {
        let source_code = "-123";
        let parse_result = parse_code_and_return_ast(source_code).unwrap();
        assert_eq!(parse_result, "(- 123.0)");
    }

    #[test]
    fn test_equality_binds_looser_than_comparison() {
        let source_code = "1 < 2 == true";
        let parse_result = parse_code_and_return_ast(source_code).unwrap();
        assert_eq!(parse_result, "(== (< 1.0 2.0) true)");
    }

    #[test]
    fn test_parse_error_on_trailing_operator() {
        let source_code = "1 +";
        let parse_result = get_parse_result(source_code);
        assert!(parse_result.is_err(), "expected parse error, got: {parse_result:?}");
    }
}
