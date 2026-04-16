#[cfg(test)]
pub mod parser_tests {
    use parser::{AstPrinter, Parser};
    use scanner::Scanner;

    #[test]
    fn test_syntax_tree() {
        let source_code = "../../test.lox";

        let tokens = Scanner::new(source_code.to_string()).scan().unwrap().0.get_tokens();

        let parser_result = Parser::new(&tokens).parse();

        println!("Parser result: {parser_result:#?}");
        assert!(parser_result.is_ok(), "expected parser to not fail here");

        println!("{:#?}", AstPrinter::print(&parser_result.unwrap()));
    }
}
