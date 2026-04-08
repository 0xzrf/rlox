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

    pub fn scan(&mut self) -> Result<(), String> {
        if self.source.is_empty() {
            self.add_token(Token::new(TokenType::EOF, 0, "".to_string()));
            self.print_tokens();
            return Ok(());
        }

        let source = self.source.clone();

        for lines in source.lines() {
            let mut line_peekable = lines.char_indices().peekable();

            while let Some((ix, c)) = line_peekable.peek() {
                let token = Self::get_token(c, ix);
                self.add_token(token);
                line_peekable.next();
            }
        }

        self.add_token(Token::new(TokenType::EOF, 0, "".to_string()));
        self.print_tokens();

        Ok(())
    }

    pub fn get_token(c: &char, ix: &usize) -> Token {
        let token_ty = match c {
            '(' => TokenType::LEFT_PAREN,
            ')' => TokenType::RIGHT_PAREN,
            '{' => TokenType::LEFT_BRACE,
            '}' => TokenType::RIGHT_BRACE,
            _ => todo!(),
        };

        Token::new(token_ty, *ix, c.to_string())
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
