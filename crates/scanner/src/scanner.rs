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

                let Some((token, to_skip)) = Self::get_token(c, ix, rest_of_line) else {
                    eprintln!("[line {}] Error: Unexpected character: {}", line_ix + 1, c);
                    line_peekable.next();
                    exit_code = 65;
                    continue;
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

    pub fn get_token(c: &char, start: &usize, rest_of_line: String) -> Option<(Token, usize)> {
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
            _ => return None,
        };


        Some((Token::new(token_ty, *start, lexeam.to_string()), to_skip))
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
