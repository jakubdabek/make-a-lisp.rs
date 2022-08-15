use std::{iter::Peekable, rc::Rc};

use crate::{
    ast::{Expr, Keyword},
    eval::builtins::list_to_hash_map,
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
    #[error("{0}")]
    LexError(String),
    #[error("{0}")]
    MapError(String),
    #[error("internal error: unknown token")]
    UnknownToken,
    #[error("internal error: {0}")]
    InternalError(String),
}

pub type ParseResult<T> = std::result::Result<T, ParseError>;

pub fn parse(s: &str) -> ParseResult<Expr> {
    let mut lexer = Lexer::new(s).peekable();
    let expr = parse_term(&mut lexer)?;
    match parse_term(&mut lexer) {
        Ok(_) => Err(ParseError::UnexpectedTerm),
        Err(ParseError::Empty) => Ok(expr),
        Err(e) => Err(e),
    }
}

fn parse_term(lexer: &mut Peekable<Lexer<'_>>) -> ParseResult<Expr> {
    let token = lexer.next().ok_or(ParseError::Empty)?;
    match token {
        Token::Atom(atom) => parse_atom(atom),
        Token::Keyword(k) => Ok(Expr::Keyword(Keyword::new(k))),
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
        Token::String(s) => Ok(Expr::String(s.into_owned())),
        Token::Error(e) => Err(ParseError::LexError(e)),
        _ => Err(ParseError::UnknownToken),
    }
}

fn parse_atom(atom: &str) -> Result<Expr, ParseError> {
    if let Ok(num) = atom.parse() {
        return Ok(Expr::Int(num));
    }

    if let Ok(num) = atom.parse() {
        return Ok(Expr::Bool(num));
    }

    if atom == "nil" {
        return Ok(Expr::Nil);
    }

    Ok(Expr::Symbol(atom.into()))
}

fn parse_special_form(lexer: &mut Peekable<Lexer<'_>>, name: &'static str) -> ParseResult<Expr> {
    let expr = parse_term(lexer)?;
    if name == "with-meta" {
        let meta = parse_term(lexer)?;
        Ok(Expr::List(vec![Expr::Symbol(name.into()), meta, expr]))
    } else {
        Ok(Expr::List(vec![Expr::Symbol(name.into()), expr]))
    }
}

fn parse_list(lexer: &mut Peekable<Lexer<'_>>, end: u8) -> ParseResult<Expr> {
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
        b'}' => list_to_hash_map(&list)
            .map(Rc::new)
            .map(Expr::Map)
            .map_err(|e| ParseError::MapError(e.to_string())),
        _ => Err(ParseError::InternalError(
            "unknown value of end for list".into(),
        )),
    }
}
