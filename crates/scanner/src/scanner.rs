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
            self.add_token(TokenType::EOF, 0, "".to_string());
        } else {
            panic!("Scanner not implemented")
        }

        for _lines in self.source.lines() {
            println!("ran a line");
        }

        self.print_tokens();

        Ok(())
    }

    pub fn print_tokens(&self) {
        for token in &self.tokens {
            println!("{}", token.to_string());
        }
    }

    pub fn add_token(&mut self, token_ty: TokenType, line: usize, lexeme: String) {
        self.tokens.push(Token::new(token_ty, line, lexeme));
    }
}
