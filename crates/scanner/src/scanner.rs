use std::fs::File;
use std::io::Read;

use interpreter_types::{Token, TokenType};

pub struct Scanner {
    source: String,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source_file: String) -> Self {
        let mut source_buffer = String::new();

        // Open the file and read its contents into the source_buffer
        let mut file = File::open(&source_file).expect("Failed to open source file");
        file.read_to_string(&mut source_buffer).expect("Failed to read source file");

        Self {
            source: source_buffer,
            tokens: Vec::new(),
        }
    }

    pub fn scan(&mut self) -> Result<i32, String> {
        if self.source.is_empty() {
            self.add_token(Token::new(TokenType::EOF, 0, "".to_string()));
            self.print_tokens();
            return Ok(0);
        }

        let source = self.source.clone();
        let mut exit_code = 0;

        for (line_ix, lines) in source.lines().enumerate() {
            let mut line_peekable = lines.char_indices().peekable();

            while let Some((ix, c)) = line_peekable.peek() {
                let rest_of_line = lines.chars().skip(*ix + 1).collect::<String>(); // skip to ix + 1 so that rest_of_line.peak() gives the element after the current value

                let (token, to_skip) = match Self::get_token(c, ix, rest_of_line) {
                    Ok((token, to_skip)) => (token, to_skip),
                    // handle the case when the line is either an error
                    Err(is_comment) => {
                        if is_comment {
                            for _ in line_peekable.by_ref() {}
                            break;
                        } else {
                            eprintln!("[line {}] Error: Unexpected character: {}", line_ix + 1, c);
                            line_peekable.next();
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

        self.add_token(Token::new(TokenType::EOF, 0, "".to_string()));
        self.print_tokens();

        Ok(exit_code)
    }

    pub fn get_token(
        c: &char,
        start: &usize,
        rest_of_line: String,
    ) -> Result<(Token, usize), bool> {
        let mut rest_peekable = rest_of_line.chars().peekable();

        let (token_ty, lexeam, to_skip) = match c {
            '(' => (TokenType::LEFT_PAREN, "(", 1),
            ')' => (TokenType::RIGHT_PAREN, ")", 1),
            '{' => (TokenType::LEFT_BRACE, "{", 1),
            '}' => (TokenType::RIGHT_BRACE, "}", 1),
            ',' => (TokenType::COMMA, ",", 1),
            '.' => (TokenType::DOT, ".", 1),
            '-' => (TokenType::MINUS, "-", 1),
            '+' => (TokenType::PLUS, "+", 1),
            ';' => (TokenType::SEMICOLON, ";", 1),
            '*' => (TokenType::STAR, "*", 1),
            '=' => {
                if rest_peekable.peek() == Some(&'=') {
                    (TokenType::EQUAL_EQUAL, "==", 2)
                } else {
                    (TokenType::EQUAL, "=", 1)
                }
            }
            '!' => {
                if rest_peekable.peek() == Some(&'=') {
                    (TokenType::BANG_EQUAL, "!=", 2)
                } else {
                    (TokenType::BANG, "!", 1)
                }
            }
            '>' => {
                if rest_peekable.peek() == Some(&'=') {
                    (TokenType::GREATER_EQUAL, ">=", 2)
                } else {
                    (TokenType::GREATER, ">", 1)
                }
            }
            '<' => {
                if rest_peekable.peek() == Some(&'=') {
                    (TokenType::LESS_EQUAL, "<=", 2)
                } else {
                    (TokenType::LESS, "<", 1)
                }
            }
            '/' => {
                if rest_peekable.peek() == Some(&'/') {
                    return Err(true);
                } else {
                    (TokenType::SLASH, "/", 1)
                }
            }
            _ => return Err(false),
        };


        Ok((Token::new(token_ty, *start, lexeam.to_string()), to_skip))
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{}", token.to_string());
        }
    }

    pub fn add_token(&mut self, token: Token) {
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
            let (tok, n) = Scanner::get_token(&ch, &0, String::new()).unwrap();
            assert_eq!(n, skip);
            assert_eq!(tok.to_string(), token_repr(ty, lex));
        }
    }

    #[test]
    fn get_token_slash_alone() {
        let (tok, n) = Scanner::get_token(&'/', &0, "x".to_string()).unwrap();
        assert_eq!(n, 1);
        assert_eq!(tok.to_string(), token_repr(TokenType::SLASH, "/"));
    }

    #[test]
    fn get_token_slash_starts_comment() {
        assert!(Scanner::get_token(&'/', &0, "/rest".to_string()).is_err());
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
            let (tok, n) = Scanner::get_token(&ch, &0, rest.to_string()).unwrap();
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
            let (tok, n) = Scanner::get_token(&ch, &0, rest.to_string()).unwrap();
            assert_eq!(n, skip);
            assert_eq!(tok.to_string(), token_repr(ty, lex));
        }
    }

    #[test]
    fn get_token_one_char_at_end_of_line() {
        let (tok, n) = Scanner::get_token(&'=', &3, String::new()).unwrap();
        assert_eq!(n, 1);
        assert_eq!(tok.to_string(), token_repr(TokenType::EQUAL, "="));
    }

    #[test]
    fn get_token_rejects_unknown() {
        assert!(matches!(Scanner::get_token(&'a', &0, String::new()), Err(false)));
        assert!(matches!(Scanner::get_token(&' ', &0, String::new()), Err(false)));
        assert!(matches!(Scanner::get_token(&'"', &0, String::new()), Err(false)));
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
        assert_eq!(s.scan().unwrap(), 65);
        let lines = s.token_lines();
        assert!(lines.iter().any(|l| l.contains("SLASH")));
        assert_eq!(lines.last().unwrap(), &token_repr(TokenType::EOF, ""));
    }

    #[test]
    fn scan_whitespace_between_tokens_not_skipped_yet() {
        // Space is not a valid token until whitespace handling exists.
        let mut s = Scanner::from_source("+ +");
        assert_eq!(s.scan().unwrap(), 65);
    }

    #[test]
    fn get_token_double_slash_lookahead_at_nonzero_offset() {
        let line = "x//";
        let ix = line.find('/').unwrap();
        let rest: String = line.chars().skip(ix + 1).collect();
        assert!(Scanner::get_token(&'/', &ix, rest).is_err());
    }
}
