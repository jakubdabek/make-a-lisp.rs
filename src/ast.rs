use std::{
    borrow::Cow,
    fmt::{self, Write},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'src> {
    Nil,
    Bool(bool),
    Int(i64),
    String(Cow<'src, str>),
    List(Vec<Expr<'src>>),
    Vector(Vec<Expr<'src>>),
    Map(Vec<Expr<'src>>),
    Symbol(Cow<'static, str>),
}

struct AsList<'a, 's>(&'a [Expr<'s>], [char; 2]);

impl fmt::Display for AsList<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(list, [start, end]) = self;
        f.write_char(*start)?;
        let mut it = list.iter();
        if let Some(first) = it.next() {
            write!(f, "{first}")?;
            for elem in it {
                write!(f, " {elem}")?;
            }
        }
        f.write_char(*end)
    }
}

impl fmt::Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Nil => f.write_str("nil"),
            Expr::Bool(b) => write!(f, "{b}"),
            Expr::Int(i) => write!(f, "{i}"),
            Expr::String(s) => write!(f, "{s:?}"),
            Expr::List(list) => write!(f, "{}", AsList(list, ['(', ')'])),
            Expr::Vector(vector) => write!(f, "{}", AsList(vector, ['[', ']'])),
            Expr::Map(map) => write!(f, "{}", AsList(map, ['{', '}'])),
            Expr::Symbol(s) => write!(f, "{s}"),
        }
    }
}
