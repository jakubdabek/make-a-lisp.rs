use std::rc::Rc;

use crate::eval::EvalFunc;

mod display;

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
    Function(EvalFunc),
    Ref(Rc<Expr>),
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

impl AsRef<Expr> for Expr {
    fn as_ref(&self) -> &Expr {
        match self {
            Expr::Ref(expr) => expr.as_ref(),
            _ => self,
        }
    }
}
