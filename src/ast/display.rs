use std::fmt::{self, Write};

use super::{Expr, Keyword};

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char(':')?;
        f.write_str(self.as_ref().trim_matches('\0'))?;
        Ok(())
    }
}

pub struct Join<'a, D>(pub &'a [D], pub &'a str);

impl<D: fmt::Display> fmt::Display for Join<'_, D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(list, delim) = self;

        let mut it = list.iter();
        if let Some(first) = it.next() {
            fmt::Display::fmt(first, f)?;
            for elem in it {
                f.write_str(delim)?;
                fmt::Display::fmt(elem, f)?;
            }
        }
        Ok(())
    }
}

pub struct Surrounded<D>(pub D, pub [char; 2]);

impl<D: fmt::Display> fmt::Display for Surrounded<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(x, [start, end]) = self;
        f.write_char(*start)?;
        fmt::Display::fmt(x, f)?;
        f.write_char(*end)?;

        Ok(())
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
            Expr::List(list) => fmt::Display::fmt(&Surrounded(Join(list, " "), ['(', ')']), f),
            Expr::Vector(vector) => {
                fmt::Display::fmt(&Surrounded(Join(vector, " "), ['[', ']']), f)
            }
            Expr::Map(map) => fmt::Display::fmt(&Surrounded(Join(map, " "), ['{', '}']), f),
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Function(_) => f.write_str("#<function>"),
            Expr::BuiltinFunction(fname) => write!(f, "{fname}"),
            Expr::Atom(a) => {
                f.write_str("(atom ")?;
                fmt::Display::fmt(&*a.borrow(), f)?;
                f.write_str(")")?;
                Ok(())
            }
            Expr::MacroExpand(_) => unreachable!(),
        }
    }
}
