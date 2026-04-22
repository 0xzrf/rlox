use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::Token;
use interpreter_types::TokenType::{self, *};

use crate::ast::{Expr, Literal, Stmt};
use crate::parser_errors::{ParserError, ParserResult};

pub struct Parser<'a> {
    tokens_peekable: Peekable<Enumerate<Iter<'a, Token>>>,
    original_tokens: Vec<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let tokens_peekable = tokens.iter().enumerate().peekable();

        Self {
            tokens_peekable,
            original_tokens: tokens.to_vec(),
        }
    }

    /// The top level interface to create the syntax tree
    pub fn parse(&mut self) -> ParserResult<Vec<Stmt>> {
        let mut stmts = Vec::new();

        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        Ok(stmts)
    }

    fn declaration(&mut self) -> ParserResult<Stmt> {
        if self.match_any(&[VAR]) {
            return self.var_declaration();
        }

        self.statement()
    }

    fn var_declaration(&mut self) -> ParserResult<Stmt> {
        let name = self.consume(&IDENTIFIER, "Expected an Identifier after var declaration")?;

        let mut initializer = None;

        if self.match_any(&[EQUAL]) {
            initializer = Some(self.expression()?);
        }

        self.consume(&SEMICOLON, "Expected a semicolon after stmt")?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> ParserResult<Stmt> {
        if self.match_any(&[IF]) {
            return self.if_statement();
        }

        if self.match_any(&[PRINT]) {
            return self.print_statment();
        }

        if self.match_any(&[WHILE]) {
            return self.while_stmt();
        }

        if self.match_any(&[FOR]) {
            return self.for_stmt();
        }

        if self.match_any(&[LEFT_BRACE]) {
            return self.block();
        }

        self.expression_stmt()
    }

    fn for_stmt(&mut self) -> ParserResult<Stmt> {
        self.consume(&LEFT_PAREN, "Expected a ( after for statment")?;

        let initializer: Option<Stmt>;

        if self.check(&SEMICOLON) {
            initializer = None;
        } else if self.match_any(&[VAR]) {
            initializer = Some(self.var_declaration()?);
        } else {
            initializer = Some(self.expression_stmt()?);
        }

        let mut condition = None;

        if !self.check(&SEMICOLON) {
            condition = Some(self.expression()?);
        }

        self.consume(&SEMICOLON, "expected a ; in the for loop")?;

        let mut increment = None;

        if !self.check(&RIGHT_PAREN) {
            increment = Some(self.expression()?);
        }

        self.consume(&RIGHT_PAREN, "Expected a ) after for clauses")?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block {
                stmts: vec![body, Stmt::Expression { expr: inc }],
            }
        }

        if condition.is_none() {
            condition = Some(Expr::Literal { value: Literal::True });
        }

        body = Stmt::While {
            body: Box::new(body),
            condition: condition.unwrap(),
        };

        if let Some(init) = initializer {
            body = Stmt::Block { stmts: vec![init, body] }
        }

        Ok(body)
    }

    fn while_stmt(&mut self) -> ParserResult<Stmt> {
        self.consume(&LEFT_PAREN, "Expected \"(\" after while")?;

        let condition = self.expression()?;

        self.consume(&RIGHT_PAREN, "Expected \")\" after while")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { body, condition })
    }

    fn if_statement(&mut self) -> ParserResult<Stmt> {
        self.consume(&LEFT_PAREN, "Expected \"(\" after if")?;

        let condition = self.expression()?;

        self.consume(&RIGHT_PAREN, "Expected \")\" after if")?;

        let then_branch = Box::new(self.statement()?);

        let mut else_branch = None;

        if self.match_any(&[ELSE]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Stmt::IfStmt { condition, then_branch, else_branch })
    }

    fn block(&mut self) -> ParserResult<Stmt> {
        let mut stmts = Vec::new();
        while !self.check(&RIGHT_BRACE) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(&RIGHT_BRACE, "Invalid block. Consider adding a }")?;

        Ok(Stmt::Block { stmts })
    }

    fn print_statment(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;

        self.consume(&SEMICOLON, "Expected the statement to end with a semicolon")?;

        Ok(Stmt::Print { expr: value })
    }

    fn expression_stmt(&mut self) -> ParserResult<Stmt> {
        let value = self.expression()?;


        self.consume(&SEMICOLON, "Expected the statement to end with a semicolon")?;

        Ok(Stmt::Expression { expr: value })
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserResult<Expr> {
        let expr = self.or()?;

        if self.match_any(&[EQUAL]) {
            let _equal = self.prev();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign { name, value: Box::new(value) });
            }
            return Err(ParserError::ParseError {
                msg: "Invalid assignment target".to_string(),
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> ParserResult<Expr> {
        let mut expr = self.and()?;

        while self.match_any(&[OR]) {
            let operator = self.prev().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserResult<Expr> {
        let mut expr = self.equality()?;

        while self.match_any(&[AND]) {
            let operator = self.prev().clone();
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParserResult<Expr> {
        let mut expr = self.comparision()?;

        while self.match_any(&[BANG_EQUAL, EQUAL_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.comparision()?;
            expr = Expr::new_binary(expr, operator, right);
        }
        Ok(expr)
    }

    fn comparision(&mut self) -> ParserResult<Expr> {
        let mut expr = self.term()?;

        while self.match_any(&[GREATER, GREATER_EQUAL, LESS, LESS_EQUAL]) {
            let operator = self.prev().clone();
            let right = self.term()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParserResult<Expr> {
        let mut expr = self.factor()?;

        while self.match_any(&[MINUS, PLUS]) {
            let operator = self.prev().clone();
            let right = self.factor()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<Expr> {
        let mut expr = self.unary()?;

        while self.match_any(&[SLASH, STAR]) {
            let operator = self.prev().clone();
            let right = self.unary()?;
            expr = Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<Expr> {
        if self.match_any(&[BANG, MINUS]) {
            let operator = self.prev().clone();
            let right = self.unary()?;
            return Ok(Expr::new_unary(operator, right));
        }

        self.call()
    }

    fn call(&mut self) -> ParserResult<Expr> {
        let mut expr = self.primary()?;

        while self.match_any(&[LEFT_PAREN]) {
            expr = self.finish_call(&expr)?;
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: &Expr) -> ParserResult<Expr> {
        let callee = Box::new(callee.clone());
        let mut args = Vec::new();

        if !self.check(&RIGHT_PAREN) {
            loop {
                if args.len() > 255 {
                    let token = self.peek().unwrap().1.clone();
                    eprintln!("{}", self.error(&token, "Can't have more then 255 arguments"));
                }

                args.push(self.expression()?);

                if self.match_any(&[COMMA]) {
                    continue;
                }
                break;
            }
        }

        let paren = self.consume(&RIGHT_PAREN, "Expected ) to end the call")?;

        Ok(Expr::Call { callee, paren, args })
    }

    fn primary(&mut self) -> ParserResult<Expr> {
        if self.match_any(&[FALSE]) {
            return Ok(Expr::new_primary(Literal::False));
        }
        if self.match_any(&[TRUE]) {
            return Ok(Expr::new_primary(Literal::True));
        }

        if self.match_any(&[NUMBER]) {
            return Ok(Expr::new_primary(Literal::Number(self.prev().literal.clone())));
        }

        if self.match_any(&[NIL]) {
            return Ok(Expr::new_primary(Literal::Nil));
        }

        if self.match_any(&[STRING]) {
            return Ok(Expr::new_primary(Literal::String(self.prev().literal.clone())));
        }

        if self.match_any(&[IDENTIFIER]) {
            return Ok(Expr::new_variable(self.prev().clone()));
        }

        if self.match_any(&[LEFT_PAREN]) {
            let expr = self.expression()?;
            self.consume(&RIGHT_PAREN, "Expect ')' after expression")?;
            return Ok(Expr::new_grouping(expr));
        }

        Err(ParserError::get_error(self.peek().unwrap().1, "Expected an expression"))
    }

    fn synchronize(&mut self) {
        self.advance();


        while !self.is_at_end() {
            if self.prev().token_ty == SEMICOLON {
                return;
            }

            match self.peek() {
                Some((_ix, token_ty)) if TokenType::is_token_starting_stmt(&token_ty.token_ty) => {
                    return;
                }
                None => return,
                _ => {}
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_ty: &TokenType, error: &str) -> Result<Token, ParserError> {
        if self.check(token_ty) {
            return Ok(self.advance().unwrap().1.clone());
        }

        let peek_token = self.peek().unwrap().1.clone();
        let parse_err = self.error(&peek_token, error);

        eprintln!("{parse_err}");
        Err(parse_err)
    }

    fn error(&mut self, token: &Token, error: &str) -> ParserError {
        ParserError::get_error(token, error)
    }

    fn match_any(&mut self, tokens: &[TokenType]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn check(&mut self, is_token: &TokenType) -> bool {
        match self.peek() {
            Some((_ix, token)) => token.token_ty == *is_token,
            None => false,
        }
    }

    fn advance(&mut self) -> Option<(usize, &Token)> {
        self.tokens_peekable.next()
    }

    fn is_at_end(&mut self) -> bool {
        matches!(self.peek(), Some((_ix, token)) if token.token_ty == EOF)
    }

    fn peek(&mut self) -> Option<&(usize, &Token)> {
        self.tokens_peekable.peek()
    }


    fn prev(&mut self) -> &Token {
        let next = self
            .tokens_peekable
            .peek()
            .map(|(ix, _)| *ix)
            .unwrap_or(self.original_tokens.len());

        let prev_ix =
            next.checked_sub(1).expect("Parser::prev() called before consuming any token");

        &self.original_tokens[prev_ix]
    }
}

#[cfg(test)]
mod tests {
    use scanner::Scanner;

    use super::Parser;
    use crate::{AstPrinter, Stmt};

    fn parse_to_ast(source: &str) -> String {
        let mut src = source.trim().to_string();
        if !src.ends_with(';') {
            src.push(';');
        }

        let tokens = Scanner::_new(src).scan(false).unwrap().0.get_tokens();
        let mut stmts = Parser::new(&tokens).parse().unwrap();
        assert_eq!(stmts.len(), 1, "expected a single statement");

        match stmts.remove(0) {
            Stmt::Expression { expr } => AstPrinter::print(&expr),
            Stmt::Print { expr } => AstPrinter::print(&expr),
            _ => panic!("test helper expects expression/print statement"),
        }
    }

    fn parse_err(source: &str) -> String {
        let mut src = source.trim().to_string();
        if !src.is_empty() && !src.ends_with(';') {
            src.push(';');
        }

        let tokens = Scanner::_new(src).scan(false).unwrap().0.get_tokens();
        let err = Parser::new(&tokens).parse().unwrap_err();
        err.to_string()
    }

    #[test]
    fn parses_number_literal() {
        assert_eq!(parse_to_ast("42"), "42.0");
    }

    #[test]
    fn parses_string_literal() {
        assert_eq!(parse_to_ast("\"hi\""), "\"hi\"");
    }

    #[test]
    fn parses_unary_bang() {
        assert_eq!(parse_to_ast("!true"), "(! true)");
    }

    #[test]
    fn parses_left_associative_term() {
        assert_eq!(parse_to_ast("10 - 3 - 2"), "(- (- 10.0 3.0) 2.0)");
    }

    #[test]
    fn parses_factor_precedence_over_term() {
        assert_eq!(parse_to_ast("8 / 2 * 3"), "(* (/ 8.0 2.0) 3.0)");
    }

    #[test]
    fn parses_nested_grouping() {
        assert_eq!(parse_to_ast("((1))"), "(group (group 1.0))");
    }

    #[test]
    fn unary_minus_binds_tighter_than_multiplication() {
        assert_eq!(parse_to_ast("-1 * 2"), "(* (- 1.0) 2.0)");
    }

    #[test]
    fn equality_is_left_associative() {
        assert_eq!(parse_to_ast("true == false == true"), "(== (== true false) true)");
    }

    #[test]
    fn errors_on_missing_right_paren() {
        let msg = parse_err("(1 + 2");
        assert!(msg.contains("Expect ')' after expression"), "unexpected error message: {msg}");
    }

    #[test]
    fn errors_on_empty_input() {
        let tokens = Scanner::_new("".to_string()).scan(false).unwrap().0.get_tokens();
        let stmts = Parser::new(&tokens).parse().unwrap();
        assert!(
            stmts.is_empty(),
            "empty input should parse as an empty program, got: {stmts:#?}"
        );
    }

    #[test]
    fn parses_chained_unary() {
        assert_eq!(parse_to_ast("!!true"), "(! (! true))");
    }

    #[test]
    fn comparison_is_left_associative() {
        assert_eq!(parse_to_ast("3 > 2 > 1"), "(> (> 3.0 2.0) 1.0)");
    }

    #[test]
    fn comparison_binds_looser_than_addition() {
        assert_eq!(parse_to_ast("1 + 2 < 4"), "(< (+ 1.0 2.0) 4.0)");
    }

    #[test]
    fn parses_mixed_grouping_and_factor() {
        assert_eq!(parse_to_ast("2 * (3 + 4)"), "(* 2.0 (group (+ 3.0 4.0)))");
    }

    #[test]
    fn errors_on_unexpected_right_paren() {
        let msg = parse_err(")");
        assert!(msg.contains("Expected an expression"), "unexpected error message: {msg}");
    }

    #[test]
    fn unary_minus_applies_to_grouping() {
        assert_eq!(parse_to_ast("-(1 + 2)"), "(- (group (+ 1.0 2.0)))");
    }

    #[test]
    fn equality_binds_looser_than_addition() {
        assert_eq!(parse_to_ast("1 + 2 == 3"), "(== (+ 1.0 2.0) 3.0)");
    }

    #[test]
    fn factor_is_left_associative() {
        assert_eq!(parse_to_ast("24 / 3 / 2"), "(/ (/ 24.0 3.0) 2.0)");
    }

    #[test]
    fn parses_string_equality() {
        assert_eq!(parse_to_ast("\"a\" == \"b\""), "(== \"a\" \"b\")");
    }

    #[test]
    fn errors_on_lone_unary_operator() {
        let msg = parse_err("!");
        assert!(msg.contains("Expected an expression"), "unexpected error message: {msg}");
    }

    #[test]
    fn parses_bang_equal() {
        assert_eq!(parse_to_ast("1 != 2"), "(!= 1.0 2.0)");
    }

    #[test]
    fn parses_less_equal() {
        assert_eq!(parse_to_ast("1 <= 2"), "(<= 1.0 2.0)");
    }

    #[test]
    fn unary_binds_tighter_than_equality() {
        assert_eq!(parse_to_ast("!false == true"), "(== (! false) true)");
    }

    #[test]
    fn parses_long_left_associative_expression() {
        assert_eq!(parse_to_ast("1 + 2 * 3 + 4"), "(+ (+ 1.0 (* 2.0 3.0)) 4.0)");
    }

    #[test]
    fn errors_on_trailing_equality_operator() {
        let msg = parse_err("true ==");
        assert!(msg.contains("Expected an expression"), "unexpected error message: {msg}");
    }

    #[test]
    fn parses_greater_equal() {
        assert_eq!(parse_to_ast("2 >= 1"), "(>= 2.0 1.0)");
    }

    #[test]
    fn parses_less_than() {
        assert_eq!(parse_to_ast("2 < 10"), "(< 2.0 10.0)");
    }

    #[test]
    fn parses_mixed_equality_chain() {
        assert_eq!(parse_to_ast("1 == 2 != 3"), "(!= (== 1.0 2.0) 3.0)");
    }

    #[test]
    fn parses_complex_grouping_and_precedence() {
        assert_eq!(
            parse_to_ast("(1 + 2) * (3 - 4 / 2)"),
            "(* (group (+ 1.0 2.0)) (group (- 3.0 (/ 4.0 2.0))))"
        );
    }

    #[test]
    fn errors_on_leading_binary_operator() {
        let msg = parse_err("+ 1");
        assert!(msg.contains("Expected an expression"), "unexpected error message: {msg}");
    }
}
