use std::iter::{Enumerate, Peekable};
use std::slice::Iter;

use interpreter_types::Token;
use interpreter_types::TokenType::{self, *};

use crate::ast::{Expr, Literal};
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
    pub fn parse(&mut self) -> ParserResult<Expr> {
        self.expression()
    }

    fn expression(&mut self) -> ParserResult<Expr> {
        self.equality()
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

        self.primary()
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

        if self.match_any(&[STRING]) {
            return Ok(Expr::new_primary(Literal::String(self.prev().literal.clone())));
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
    use crate::AstPrinter;

    fn parse_to_ast(source: &str) -> String {
        let tokens = Scanner::_new(source.to_string()).scan(false).unwrap().0.get_tokens();
        let expr = Parser::new(&tokens).parse().unwrap();
        AstPrinter::print(&expr)
    }

    fn parse_err(source: &str) -> String {
        let tokens = Scanner::_new(source.to_string()).scan(false).unwrap().0.get_tokens();
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
        let msg = parse_err("");
        assert!(msg.contains("Expected an expression"), "unexpected error message: {msg}");
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
