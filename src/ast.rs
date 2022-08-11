use std::{cell::RefCell, rc::Rc};

use crate::environment::Env;

pub mod display;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Nil,
    Bool(bool),
    Int(i64),
    String(String),
    List(Vec<Expr>),
    Vector(Vec<Expr>),
    Map(Vec<Expr>),
    Symbol(Rc<str>),
    Keyword(Keyword),
    Function(Function),
    BuiltinFunction(&'static str),
    Atom(Rc<RefCell<Expr>>),
}

impl Expr {
    pub const NIL: Expr = Expr::Nil;

    pub fn atom(e: Expr) -> Expr {
        Self::Atom(Rc::new(RefCell::new(e)))
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Expr::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_symbol(&self) -> Option<&str> {
        match self {
            Expr::Symbol(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_builtin(&self) -> Option<&'static str> {
        match self {
            Expr::BuiltinFunction(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_func_name(&self) -> Option<&str> {
        self.as_symbol().or_else(|| self.as_builtin())
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Expr::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_list_like(&self) -> Option<&[Expr]> {
        match self {
            Expr::List(l) | Expr::Vector(l) => Some(l),
            _ => None,
        }
    }

    pub fn into_list_like(self) -> Result<Vec<Expr>, Self> {
        match self {
            Expr::List(l) | Expr::Vector(l) => Ok(l),
            _ => Err(self),
        }
    }

    pub fn lenient_eq(&self, other: &Self) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            (Expr::List(a) | Expr::Vector(a), Expr::List(b) | Expr::Vector(b)) => {
                a.len() == b.len() && a.iter().zip(b).all(|(a, b)| a.lenient_eq(b))
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub bindings: Vec<String>,
    pub varargs: Option<String>,
    pub expr: Rc<Expr>,
    pub closure: Env,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keyword(Rc<str>);

impl Keyword {
    pub fn new(s: &str) -> Self {
        Self(format!("\0{s}\0").into())
    }
}

impl AsRef<str> for Keyword {
    fn as_ref(&self) -> &str {
        &*self.0
    }
}
