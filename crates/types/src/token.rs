use crate::TokenType;

pub struct Token {
    token_ty: TokenType,
    line: usize,
    line_offset: usize,
    lexeme: String,
    literal: String,
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.token_ty == other.token_ty
    }
}

impl Token {
    pub fn new(
        token_ty: TokenType,
        line: usize,
        lexeme: String,
        line_offset: usize,
        literal: String,
    ) -> Self {
        Self {
            token_ty,
            line,
            lexeme,
            line_offset,
            literal,
        }
    }

    pub fn to_string(&self) -> String {
        format!("{:#?} {} {}", self.token_ty, self.lexeme, self.literal)
    }
}
