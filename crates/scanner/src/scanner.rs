use std::fs::File;
use std::io::Read;
use std::iter::Peekable;
use std::str::Chars;

use interpreter_types::{Token, TokenType};

use crate::{NULL, ScannerError};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
}

type MaybeToken = Option<(Token, usize)>;
type GetTokenErr = (bool, Option<usize>, ScannerError);

impl Scanner {
    pub fn new(source_file: String) -> Self {
        let mut source_buffer = String::new();

        let mut file = File::open(&source_file).expect("Failed to open source file");
        file.read_to_string(&mut source_buffer).expect("Failed to read source file");

        Self {
            source: source_buffer,
            tokens: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Result<i32, String> {
        if self.source.is_empty() {
            self.add_token(Token::new(TokenType::EOF, 0, "".to_string(), 0, NULL.to_string()));
            self.print_tokens();
            return Ok(0);
        }

        let source = self.source.clone();
        // println!("source: {source}");
        let mut exit_code = 0;

        for (line_ix, lines) in source.lines().enumerate() {
            let mut line_peekable = lines.char_indices().peekable();

            while let Some((ix, c)) = line_peekable.peek() {
                let rest_of_line = lines.chars().skip(*ix + 1).collect::<String>(); // skip to ix + 1 so that rest_of_line.peak() gives the element after the current value

                let (token, to_skip) = match Self::get_token(c, ix, &line_ix, rest_of_line) {
                    Ok(Some((token, to_skip))) => (token, to_skip),
                    Ok(None) => {
                        // this is the case when the character token needs to be skipped
                        line_peekable.next();
                        continue;
                    }
                    // handle the case when the line is either an error
                    Err((is_comment, to_skip, err_msg)) => {
                        if is_comment {
                            for _ in line_peekable.by_ref() {}
                            break;
                        } else {
                            eprintln!("[line {}] Error: {}", line_ix + 1, err_msg);
                            let to_skip = to_skip.unwrap_or(1);

                            for _ in 0..to_skip {
                                line_peekable.next();
                            }

                            exit_code = 65;
                        }
                        continue;
                    }
                };
                self.add_token(token);

                for _ in 0..to_skip {
                    line_peekable.next();
                }
            }
        }

        self.add_token(Token::new(TokenType::EOF, 0, "".to_string(), 0, NULL.to_string()));
        self.print_tokens();

        Ok(exit_code)
    }

    fn get_token(
        c: &char,
        line_offset: &usize,
        line_ix: &usize,
        rest_of_line: String,
    ) -> Result<MaybeToken, GetTokenErr> {
        let mut rest_peekable = rest_of_line.chars().peekable();

        let (token_ty, lexeam, to_skip, literal) = match c {
            // Single tokens
            '(' => (TokenType::LEFT_PAREN, "(".to_string(), 1, "null".to_string()),
            ')' => (TokenType::RIGHT_PAREN, ")".to_string(), 1, "null".to_string()),
            '{' => (TokenType::LEFT_BRACE, "{".to_string(), 1, "null".to_string()),
            '}' => (TokenType::RIGHT_BRACE, "}".to_string(), 1, "null".to_string()),
            ',' => (TokenType::COMMA, ",".to_string(), 1, "null".to_string()),
            '.' => (TokenType::DOT, ".".to_string(), 1, "null".to_string()),
            '-' => (TokenType::MINUS, "-".to_string(), 1, "null".to_string()),
            '+' => (TokenType::PLUS, "+".to_string(), 1, "null".to_string()),
            ';' => (TokenType::SEMICOLON, ";".to_string(), 1, "null".to_string()),
            '*' => (TokenType::STAR, "*".to_string(), 1, "null".to_string()),

            // Single-double tokens
            '=' => {
                if Self::match_next(&mut rest_peekable, '=') {
                    (TokenType::EQUAL_EQUAL, "==".to_string(), 2, "null".to_string())
                } else {
                    (TokenType::EQUAL, "=".to_string(), 1, "null".to_string())
                }
            }
            '!' => {
                if Self::match_next(&mut rest_peekable, '=') {
                    (TokenType::BANG_EQUAL, "!=".to_string(), 2, "null".to_string())
                } else {
                    (TokenType::BANG, "!".to_string(), 1, "null".to_string())
                }
            }
            '>' => {
                if Self::match_next(&mut rest_peekable, '=') {
                    (TokenType::GREATER_EQUAL, ">=".to_string(), 2, "null".to_string())
                } else {
                    (TokenType::GREATER, ">".to_string(), 1, "null".to_string())
                }
            }
            '<' => {
                if Self::match_next(&mut rest_peekable, '=') {
                    (TokenType::LESS_EQUAL, "<=".to_string(), 2, "null".to_string())
                } else {
                    (TokenType::LESS, "<".to_string(), 1, "null".to_string())
                }
            }
            '/' => {
                if Self::match_next(&mut rest_peekable, '/') {
                    return Err((true, None, ScannerError::UnexpectedCharacter { c: '/' }));
                } else {
                    (TokenType::SLASH, "/".to_string(), 1, "null".to_string())
                }
            }

            // Handle whitespace, nl and tab
            ' ' | '\t' | '\r' => {
                // println!("got a space character");
                return Ok(None);
            }

            // Handle literal string
            '"' => {
                if !rest_of_line.contains("\"") {
                    return Err((
                        false,
                        Some(rest_of_line.len() + 1),
                        ScannerError::UnterminatedString,
                    ));
                }

                let lit = Self::get_string_till(&mut rest_peekable, '"');
                let lit_lexeme = format!("\"{}\"", lit);

                (TokenType::STRING, lit_lexeme, lit.len() + 2, lit)
            }

            // Handle literal Integers (leading digit is `c`; further digits live in `rest_peekable`)
            digit if digit.is_numeric() => {
                let mut lit = String::new();
                lit.push(*digit);

                lit.push_str(&Self::get_numeric_string(&mut rest_peekable));

                let lit_lexeme = lit.clone();

                let len = lit.len();

                if !lit.contains(".") {
                    lit.push_str(".0");
                } else {
                    // Remove all the trailing '0' in lit (after the decimal point),
                    // unless it's just a single '0' after the '.', i.e., leave "123.0" as is,
                    // but for cases like "123.1000" make it "123.1"; for "123.0000", "123.0".
                    if let Some(dot_pos) = lit.find('.') {
                        // Split into integer and fraction part
                        let (left, right) = lit.split_at(dot_pos + 1); // right starts with fraction
                        // remove trailing zeros from fraction
                        let right_trimmed = right.trim_end_matches('0');
                        // if nothing left after the dot, keep one '0'
                        let final_lit = if right_trimmed.is_empty() {
                            format!("{}0", left)
                        } else {
                            format!("{}{}", left, right_trimmed)
                        };
                        lit = final_lit;
                    }
                }

                (TokenType::NUMBER, lit_lexeme, len, lit)
            }
            alpha_num if alpha_num.is_alphanumeric() | alpha_num.eq(&'_') => {
                let mut lexeme = String::new();
                lexeme.push(*c);

                lexeme.push_str(&Self::get_identifier_string(&mut rest_peekable));
                let lexeme_len = lexeme.len();
                (TokenType::IDENTIFIER, lexeme, lexeme_len, "null".to_string())
            }
            _ => return Err((false, None, ScannerError::UnexpectedCharacter { c: *c })),
        };

        Ok(Some((Token::new(token_ty, *line_ix, lexeam, *line_offset, literal), to_skip)))
    }

    #[inline(always)]
    fn match_next(input_peekable: &mut Peekable<Chars>, c: char) -> bool {
        Some(c) == input_peekable.peek().copied()
    }

    #[inline(always)]
    fn get_string_till(input_peekable: &mut Peekable<Chars>, till: char) -> String {
        let mut string_lit_buf = String::new();
        while let Some(c) = input_peekable.next_if(|c| c.ne(&till)) {
            string_lit_buf.push(c);
        }

        string_lit_buf
    }

    #[inline(always)]
    fn get_identifier_string(input_peekable: &mut Peekable<Chars>) -> String {
        let mut string_lit_buf = String::new();
        while let Some(c) = input_peekable.peek().copied() {
            if !c.is_alphanumeric() && c.ne(&'_') {
                break;
            }

            string_lit_buf.push(c);
            input_peekable.next();
        }

        string_lit_buf
    }

    /// Continues a numeric lexeme from `input_peekable` (does not include the already-consumed
    /// leading digit). Uses `peek().copied()` so we never store `&char` from `peek()` across
    /// `next()` calls, which would trigger borrow-checker errors.
    fn get_numeric_string(input_peekable: &mut Peekable<Chars>) -> String {
        let mut suffix = String::new();
        while let Some(c) = input_peekable.peek().copied() {
            if !c.is_numeric() && c.ne(&'.') {
                break;
            }

            suffix.push(c);
            input_peekable.next();
        }
        suffix
    }

    fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{}", token.to_string());
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

#[cfg(test)]
impl Scanner {
    fn from_source(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            tokens: Vec::new(),
        }
    }

    fn token_lines(&self) -> Vec<String> {
        self.tokens.iter().map(|t| t.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_repr(ty: TokenType, lexeme: &str) -> String {
        format!("{:#?} {} null", ty, lexeme)
    }

    fn token_repr_lit(ty: TokenType, lexeme: &str, literal: &str) -> String {
        format!("{:#?} {} {}", ty, lexeme, literal)
    }

    #[test]
    fn get_token_single_char_delimiters() {
        let cases = [
            ('(', TokenType::LEFT_PAREN, "(", 1),
            (')', TokenType::RIGHT_PAREN, ")", 1),
            ('{', TokenType::LEFT_BRACE, "{", 1),
            ('}', TokenType::RIGHT_BRACE, "}", 1),
            (',', TokenType::COMMA, ",", 1),
            ('.', TokenType::DOT, ".", 1),
            ('-', TokenType::MINUS, "-", 1),
            ('+', TokenType::PLUS, "+", 1),
            (';', TokenType::SEMICOLON, ";", 1),
            ('*', TokenType::STAR, "*", 1),
        ];
        for (ch, ty, lex, skip) in cases {
            let (tok, n) = Scanner::get_token(&ch, &0, &0, String::new()).unwrap().unwrap();
            assert_eq!(n, skip);
            assert_eq!(tok.to_string(), token_repr(ty, lex));
        }
    }

    #[test]
    fn get_token_slash_alone() {
        let (tok, n) = Scanner::get_token(&'/', &0, &0, "x".to_string()).unwrap().unwrap();
        assert_eq!(n, 1);
        assert_eq!(tok.to_string(), token_repr(TokenType::SLASH, "/"));
    }

    #[test]
    fn get_token_slash_starts_comment() {
        assert!(Scanner::get_token(&'/', &0, &0, "/rest".to_string()).is_err());
    }

    #[test]
    fn get_token_two_char_operators() {
        let cases = [
            ('=', "=x", TokenType::EQUAL_EQUAL, "==", 2),
            ('!', "=x", TokenType::BANG_EQUAL, "!=", 2),
            ('>', "=x", TokenType::GREATER_EQUAL, ">=", 2),
            ('<', "=x", TokenType::LESS_EQUAL, "<=", 2),
        ];
        for (ch, rest, ty, lex, skip) in cases {
            let (tok, n) = Scanner::get_token(&ch, &0, &0, rest.to_string()).unwrap().unwrap();
            assert_eq!(n, skip, "lexeme {lex}");
            assert_eq!(tok.to_string(), token_repr(ty, lex));
        }
    }

    #[test]
    fn get_token_one_char_when_second_not_equals() {
        let singles = [
            ('=', TokenType::EQUAL, "=", 1, "x"),
            ('!', TokenType::BANG, "!", 1, "x"),
            ('>', TokenType::GREATER, ">", 1, "x"),
            ('<', TokenType::LESS, "<", 1, "x"),
        ];
        for (ch, ty, lex, skip, rest) in singles {
            let (tok, n) = Scanner::get_token(&ch, &0, &0, rest.to_string()).unwrap().unwrap();
            assert_eq!(n, skip);
            assert_eq!(tok.to_string(), token_repr(ty, lex));
        }
    }

    #[test]
    fn get_token_one_char_at_end_of_line() {
        let (tok, n) = Scanner::get_token(&'=', &3, &0, String::new()).unwrap().unwrap();
        assert_eq!(n, 1);
        assert_eq!(tok.to_string(), token_repr(TokenType::EQUAL, "="));
    }

    #[test]
    fn get_token_rejects_unknown() {
        assert!(matches!(
            Scanner::get_token(&'a', &0, &0, String::new()),
            Err((false, None, ScannerError::UnexpectedCharacter { c: 'a' }))
        ));
        assert!(matches!(
            Scanner::get_token(&'@', &0, &0, String::new()),
            Err((false, None, ScannerError::UnexpectedCharacter { c: '@' }))
        ));
    }

    #[test]
    fn scan_empty_source_eof_only() {
        let mut s = Scanner::from_source("");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(s.token_lines(), vec![token_repr(TokenType::EOF, "")]);
    }

    #[test]
    fn scan_parentheses() {
        let mut s = Scanner::from_source("()");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::LEFT_PAREN, "("),
                token_repr(TokenType::RIGHT_PAREN, ")"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_line_comment_ignores_rest_of_line() {
        let mut s = Scanner::from_source("()// Comment");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::LEFT_PAREN, "("),
                token_repr(TokenType::RIGHT_PAREN, ")"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_comment_only_line_then_tokens() {
        let mut s = Scanner::from_source("// only comment\n()");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::LEFT_PAREN, "("),
                token_repr(TokenType::RIGHT_PAREN, ")"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_comment_stops_at_line_break() {
        let mut s = Scanner::from_source("// a\n+");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![token_repr(TokenType::PLUS, "+"), token_repr(TokenType::EOF, ""),]
        );
    }

    #[test]
    fn scan_unexpected_character_sets_exit_65_and_continues() {
        let mut s = Scanner::from_source("(@)");
        assert_eq!(s.scan().unwrap(), 65);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::LEFT_PAREN, "("),
                token_repr(TokenType::RIGHT_PAREN, ")"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_double_char_operators_individually() {
        for (src, ty, lex) in [
            ("!=", TokenType::BANG_EQUAL, "!="),
            ("==", TokenType::EQUAL_EQUAL, "=="),
            ("<=", TokenType::LESS_EQUAL, "<="),
            (">=", TokenType::GREATER_EQUAL, ">="),
        ] {
            let mut s = Scanner::from_source(src);
            assert_eq!(s.scan().unwrap(), 0, "source {src:?}");
            assert_eq!(s.token_lines(), vec![token_repr(ty, lex), token_repr(TokenType::EOF, "")]);
        }
    }

    #[test]
    fn scan_braces_commas_and_dots() {
        let mut s = Scanner::from_source("{,}.");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::LEFT_BRACE, "{"),
                token_repr(TokenType::COMMA, ","),
                token_repr(TokenType::RIGHT_BRACE, "}"),
                token_repr(TokenType::DOT, "."),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_arithmetic_punctuation() {
        let mut s = Scanner::from_source("+-;*");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::PLUS, "+"),
                token_repr(TokenType::MINUS, "-"),
                token_repr(TokenType::SEMICOLON, ";"),
                token_repr(TokenType::STAR, "*"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_chained_comparison_without_bang() {
        // `==<=>=` — no leading `!=`, so `rest_of_line` / peek behavior is easy to reason about.
        let mut s = Scanner::from_source("==<=>=");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::EQUAL_EQUAL, "=="),
                token_repr(TokenType::LESS_EQUAL, "<="),
                token_repr(TokenType::GREATER_EQUAL, ">="),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_slash_not_comment_when_single() {
        let mut s = Scanner::from_source("1/2");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr_lit(TokenType::NUMBER, "1", "1.0"),
                token_repr(TokenType::SLASH, "/"),
                token_repr_lit(TokenType::NUMBER, "2", "2.0"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn get_token_number_single_digit() {
        let (tok, n) = Scanner::get_token(&'5', &0, &0, String::new()).unwrap().unwrap();
        assert_eq!(n, 1);
        assert_eq!(tok.to_string(), token_repr_lit(TokenType::NUMBER, "5", "5.0"));
    }

    #[test]
    fn get_token_number_multi_digit_and_skip() {
        let (tok, n) = Scanner::get_token(&'1', &0, &0, "234+".to_string()).unwrap().unwrap();
        assert_eq!(n, 4);
        assert_eq!(tok.to_string(), token_repr_lit(TokenType::NUMBER, "1234", "1234.0"));
    }

    #[test]
    fn get_token_number_decimal() {
        let (tok, n) = Scanner::get_token(&'3', &0, &0, ".14)".to_string()).unwrap().unwrap();
        assert_eq!(n, 4);
        assert_eq!(tok.to_string(), token_repr_lit(TokenType::NUMBER, "3.14", "3.14"));
    }

    #[test]
    fn get_token_number_trims_fraction_trailing_zeros() {
        let (tok, n) = Scanner::get_token(&'1', &0, &0, ".2000x".to_string()).unwrap().unwrap();
        assert_eq!(n, 6);
        assert_eq!(tok.to_string(), token_repr_lit(TokenType::NUMBER, "1.2000", "1.2"));
    }

    #[test]
    fn get_token_number_fraction_all_zeros_becomes_dot_zero() {
        let (tok, n) = Scanner::get_token(&'9', &0, &0, ".000".to_string()).unwrap().unwrap();
        assert_eq!(n, 5);
        assert_eq!(tok.to_string(), token_repr_lit(TokenType::NUMBER, "9.000", "9.0"));
    }

    #[test]
    fn scan_numbers_separated_by_whitespace() {
        let mut s = Scanner::from_source("42  99.5 ;");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr_lit(TokenType::NUMBER, "42", "42.0"),
                token_repr_lit(TokenType::NUMBER, "99.5", "99.5"),
                token_repr(TokenType::SEMICOLON, ";"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_whitespace_between_tokens_not_skipped_yet() {
        let mut s = Scanner::from_source("+ +");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr(TokenType::PLUS, "+"),
                token_repr(TokenType::PLUS, "+"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_ignores_spaces_tabs_and_carriage_returns() {
        let mut s = Scanner::from_source(" \t \r\t  \r");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(s.token_lines(), vec![token_repr(TokenType::EOF, "")]);
    }

    #[test]
    fn scan_ignores_whitespace_across_lines() {
        // Includes tabs and Windows-style CRLF. `lines()` strips line terminators, but `\t` remains.
        let mut s = Scanner::from_source(" \t\r\n\t \n  \r\n");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(s.token_lines(), vec![token_repr(TokenType::EOF, "")]);
    }

    #[test]
    fn scan_string_literal_tokenizes_lexeme_and_literal() {
        let mut s = Scanner::from_source("\"hello\"");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr_lit(TokenType::STRING, "\"hello\"", "hello"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_string_literal_allows_spaces_and_tabs_inside() {
        let mut s = Scanner::from_source("\"a b\tc\"");
        assert_eq!(s.scan().unwrap(), 0);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr_lit(TokenType::STRING, "\"a b\tc\"", "a b\tc"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_unterminated_string_line_no_closing_quote() {
        let mut s = Scanner::from_source("\"hello");
        assert_eq!(s.scan().unwrap(), 65);
        assert_eq!(s.token_lines(), vec![token_repr(TokenType::EOF, "")]);
    }

    #[test]
    fn scan_unterminated_string_only_opening_quote() {
        let mut s = Scanner::from_source("\"");
        assert_eq!(s.scan().unwrap(), 65);
        assert_eq!(s.token_lines(), vec![token_repr(TokenType::EOF, "")]);
    }

    #[test]
    fn scan_unterminated_string_after_valid_literal_and_semicolon() {
        let mut s = Scanner::from_source("\"baz\" ; \"unterminated");
        assert_eq!(s.scan().unwrap(), 65);
        assert_eq!(
            s.token_lines(),
            vec![
                token_repr_lit(TokenType::STRING, "\"baz\"", "baz"),
                token_repr(TokenType::SEMICOLON, ";"),
                token_repr(TokenType::EOF, ""),
            ]
        );
    }

    #[test]
    fn scan_unterminated_string_after_another_string() {
        let mut s = Scanner::from_source("\"a\" \"b");
        assert_eq!(s.scan().unwrap(), 65);
        assert_eq!(
            s.token_lines(),
            vec![token_repr_lit(TokenType::STRING, "\"a\"", "a"), token_repr(TokenType::EOF, ""),]
        );
    }

    #[test]
    fn get_token_unterminated_string_reports_error() {
        let rest = "no-closing-quote".to_string();
        let expected_skip = Some(rest.len() + 1);
        assert!(matches!(
            Scanner::get_token(&'"', &0, &0, rest),
            Err((false, skip, ScannerError::UnterminatedString)) if skip == expected_skip
        ));
    }

    #[test]
    fn get_token_unterminated_only_quote_after_is_empty() {
        assert!(matches!(
            Scanner::get_token(&'"', &0, &0, String::new()),
            Err((false, Some(1), ScannerError::UnterminatedString))
        ));
    }
}
