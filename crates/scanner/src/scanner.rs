use std::fs::File;
use std::io::Read;

use interpreter_types::Token;

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
        println!("{}", self.source);
        Ok(())
    }
}
