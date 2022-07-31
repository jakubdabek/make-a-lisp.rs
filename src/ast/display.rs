use std::fmt::{self, Write};

use super::{Expr, Keyword};

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(':')?;
        f.write_str(self.as_ref().trim_matches('\0'))
    }
}

struct AsList<'a>(&'a [Expr], [char; 2]);

impl fmt::Display for AsList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(list, [start, end]) = self;
        f.write_char(*start)?;
        let mut it = list.iter();
        if let Some(first) = it.next() {
            fmt::Display::fmt(first, f)?;
            for elem in it {
                f.write_char(' ')?;
                fmt::Display::fmt(elem, f)?;
            }
        }
        f.write_char(*end)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_ref() {
            Expr::Nil => f.write_str("nil"),
            Expr::Bool(b) => write!(f, "{b}"),
            Expr::Int(i) => write!(f, "{i}"),
            Expr::String(s) => {
                if f.alternate() {
                    // prints escaped strings
                    fmt::Debug::fmt(s, f)
                } else {
                    fmt::Display::fmt(s, f)
                }
            }
            Expr::Keyword(k) => fmt::Display::fmt(k, f),
            Expr::List(list) => fmt::Display::fmt(&AsList(list, ['(', ')']), f),
            Expr::Vector(vector) => fmt::Display::fmt(&AsList(vector, ['[', ']']), f),
            Expr::Map(map) => fmt::Display::fmt(&AsList(map, ['{', '}']), f),
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Function(_) => f.write_str("#<function>"),
            Expr::Ref(_) => unreachable!(),
        }
    }
}
