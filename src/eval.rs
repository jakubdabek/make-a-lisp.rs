use std::{cmp, fmt, rc::Rc};

use crate::{ast::Expr, environment::Env};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("invalid function name: {0}")]
    InvalidFunctionName(String),
    #[error("invalid number of arguments")]
    InvalidArgumentCount,
    #[error("invalid function: {0}")]
    InvalidFunction(String),
    #[error("invalid function arguments ({0}, {1})")]
    InvalidArguments(String, String),
    #[error("unknown symbol: {0}")]
    UnknownSymbol(Rc<str>),
}

pub type EvalResult<T> = std::result::Result<T, EvalError>;

#[derive(Clone)]
pub struct EvalFunc(pub for<'a> fn(&Expr, &Expr) -> EvalResult<Expr>);

impl fmt::Debug for EvalFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EvalFunc").finish()
    }
}

impl cmp::PartialEq for EvalFunc {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

pub fn eval(expr: &Expr, env: &Env) -> EvalResult<Expr> {
    match expr {
        Expr::Symbol(sym) => match env.get(&**sym) {
            Some(f) => Ok(f),
            None => Err(EvalError::UnknownSymbol(sym.clone())),
        },
        Expr::List(v) => eval_list(v, env),
        Expr::Vector(v) => Ok(Expr::Vector(
            v.iter().map(|e| eval(e, env)).collect::<EvalResult<_>>()?,
        )),
        Expr::Map(m) => Ok(Expr::Map(
            m.iter().map(|e| eval(e, env)).collect::<EvalResult<_>>()?,
        )),
        expr => Ok(expr.clone()),
    }
}

fn eval_list(exprs: &[Expr], env: &Env) -> EvalResult<Expr> {
    if exprs.is_empty() {
        return Ok(Expr::List(vec![]));
    }

    if exprs.len() != 3 {
        return Err(EvalError::InvalidArgumentCount);
    }

    let mut exprs = exprs.iter();
    let [name, a, b] = [(); 3].map(|_| exprs.next().unwrap());
    let name = eval(name, env)?;

    let f = match &name {
        Expr::Function(f) => f,
        _ => return Err(EvalError::InvalidFunctionName(name.to_string())),
    };

    let a = eval(a, env)?;
    let b = eval(b, env)?;

    (f.0)(&a, &b)
}

fn number_args(a: &Expr, b: &Expr) -> EvalResult<(i64, i64)> {
    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Ok((*a, *b)),
        _ => Err(EvalError::InvalidArguments(a.to_string(), b.to_string())),
    }
}

macro_rules! def_arithmetic {
    ($($fn:ident $op:tt),+ $(,)?) => {
        $(
        pub fn $fn<'s>(a: &Expr, b: &Expr) -> EvalResult<Expr> {
            let (a, b) = number_args(a, b)?;
            Ok(Expr::Int(a $op b))
        }
        )+
    };
}

def_arithmetic! {
    eval_add +,
    eval_sub -,
    eval_mul *,
    eval_div /,
}
