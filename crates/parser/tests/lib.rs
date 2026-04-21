#[cfg(test)]
pub mod parser_tests {
    use parser::{AstPrinter, Expr, Parser, ParserResult, Stmt};
    use scanner::Scanner;

    fn get_parse_result(source_code: &str) -> ParserResult<Vec<Stmt>> {
        let tokens = Scanner::_new(source_code.to_string()).scan(false).unwrap().0.get_tokens();

        Parser::new(&tokens).parse()
    }

    fn parse_single_expr_stmt(source_code: &str) -> Option<Expr> {
        let mut src = source_code.trim().to_string();
        if !src.ends_with(';') {
            src.push(';');
        }

        let mut stmts = get_parse_result(&src).ok()?;
        if stmts.len() != 1 {
            return None;
        }

        match stmts.remove(0) {
            Stmt::Expression { expr } => Some(expr),
            Stmt::Print { .. }
            | Stmt::Var { .. }
            | Stmt::Block { .. }
            | Stmt::IfStmt { .. }
            | Stmt::While { .. } => None,
        }
    }

    fn parse_code_and_return_ast(source_code: &str) -> Option<String> {
        let parsed_expr = parse_single_expr_stmt(source_code)?;

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

        let stmts = parser_result.unwrap();
        let printable = stmts
            .iter()
            .map(|stmt| match stmt {
                Stmt::Expression { expr } => AstPrinter::print(expr),
                Stmt::Print { expr } => format!("(print {})", AstPrinter::print(expr)),
                Stmt::Var { .. } => "(var ...)".to_string(),
                Stmt::Block { .. } => "(block ...)".to_string(),
                Stmt::IfStmt { .. } => "(if ...)".to_string(),
                Stmt::While { .. } => "(while ...)".to_string(),
            })
            .collect::<Vec<_>>();

        println!("{printable:#?}");
    }
}
