use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<'src> {
    Nil,
    Bool(bool),
    Int(i64),
    String(Cow<'src, str>),
    List(Vec<Expr<'src>>),
    Vector(Vec<Expr<'src>>),
    Symbol(Cow<'static, str>),
}
