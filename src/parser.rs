use crate::lexer::{Lexer, Token};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {}

pub type Expr<'a> = Vec<Token<'a>>;

pub fn parse(s: &str) -> Result<Expr<'_>, ParseError> {
    let lexer = Lexer::new(s);
    Ok(lexer.collect())
}
