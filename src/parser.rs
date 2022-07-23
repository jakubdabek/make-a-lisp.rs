use std::iter::Peekable;

use crate::{
    ast::Expr,
    lexer::{Lexer, Token},
};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("empty")]
    Empty,
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("too many terms")]
    UnexpectedTerm,
    #[error("unmatched delimiter: '{0}'")]
    UnmatchedDelimiter(char),
    #[error("internal error: unknown token")]
    UnknownToken,
    #[error("internal error: {0}")]
    InternalError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub fn parse(s: &str) -> ParseResult<Expr<'_>> {
    let mut lexer = Lexer::new(s).peekable();
    let expr = parse_term(&mut lexer)?;
    match parse_term(&mut lexer) {
        Ok(_) => Err(ParseError::UnexpectedTerm),
        Err(ParseError::Empty) => Ok(expr),
        Err(e) => Err(e),
    }
}

fn parse_term<'s>(lexer: &mut Peekable<Lexer<'s>>) -> ParseResult<Expr<'s>> {
    let token = lexer.next().ok_or(ParseError::Empty)?;
    match token {
        Token::Atom(atom) if atom.bytes().all(|b| b.is_ascii_digit()) => {
            Ok(Expr::Int(atom.parse().unwrap()))
        }
        Token::Atom(atom) => Ok(Expr::Symbol(atom.to_owned().into())),
        Token::Special([b'~', b'@']) => parse_special_form(lexer, "splice-unquote"),
        Token::Special([b'~', b'\0']) => parse_special_form(lexer, "unquote"),
        Token::Special([b'`', _]) => parse_special_form(lexer, "quasiquote"),
        Token::Special([b'^', _]) => parse_special_form(lexer, "with-meta"),
        Token::Special([b'@', _]) => parse_special_form(lexer, "deref"),
        Token::Special([b'\'', _]) => parse_special_form(lexer, "quote"),
        Token::Special([b'(', _]) => parse_list(lexer, b')'),
        Token::Special([b'[', _]) => parse_list(lexer, b']'),
        Token::Special([b'{', _]) => parse_list(lexer, b'}'),
        Token::Special([b')', _]) => Err(ParseError::UnmatchedDelimiter(')')),
        Token::Special([b']', _]) => Err(ParseError::UnmatchedDelimiter(']')),
        Token::Special([b'}', _]) => Err(ParseError::UnmatchedDelimiter('}')),
        Token::String(s) => Ok(Expr::String(s)),
        _ => Err(ParseError::UnknownToken),
    }
}

fn parse_special_form<'s>(
    lexer: &mut Peekable<Lexer<'s>>,
    name: &'static str,
) -> ParseResult<Expr<'s>> {
    let target = parse_term(lexer)?;
    Ok(Expr::List(vec![Expr::Symbol(name.into()), target]))
}

fn parse_list<'s>(lexer: &mut Peekable<Lexer<'s>>, end: u8) -> ParseResult<Expr<'s>> {
    let mut list = vec![];
    loop {
        match lexer.peek().ok_or(ParseError::UnexpectedEof)? {
            Token::Special([s, _]) if *s == end => {
                lexer.next();
                break;
            }
            _ => list.push(parse_term(&mut *lexer)?),
        }
    }

    match end {
        b')' => Ok(Expr::List(list)),
        b']' => Ok(Expr::Vector(list)),
        b'}' => todo!(),
        _ => Err(ParseError::InternalError(
            "unknown value of end for list".into(),
        )),
    }
}
