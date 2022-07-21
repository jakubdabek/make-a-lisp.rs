use crate::lexer::{Lexer, Token};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {}

pub type Expr<'a> = Vec<Token<'a>>;

pub fn parse(s: &str) -> Result<Expr<'_>, ParseError> {
    let mut lexer = Lexer::new(s);
    Ok(std::iter::from_fn(|| lexer.next()).collect())
}
