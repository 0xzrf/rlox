use interpreter_types::Token;

pub struct CompileTimeError {
    pub token: Token,
    pub message: &'static str,
}
