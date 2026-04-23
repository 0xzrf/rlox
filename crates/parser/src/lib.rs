mod parser;
pub use parser::Parser;
mod ast;
mod parser_errors;

pub use ast::*;
pub use parser_errors::ParserResult;

pub mod interpret;
pub use interpret::Interpret;

mod env;

mod resolver;
pub use resolver::Resolver;

mod errors;
