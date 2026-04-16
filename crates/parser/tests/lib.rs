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
    #[ignore = "internal test function"]
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
}
