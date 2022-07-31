use std::{cmp, fmt, rc::Rc};

use crate::{
    ast::Expr,
    environment::{Env, Environment},
};

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
    #[error("'{0}' not found")]
    UnknownSymbol(Rc<str>),
    #[error("invalid variable name: {0}")]
    InvalidVariableName(String),
    #[error("invalid variables for let*")]
    InvalidLetVariables,
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
    match expr.as_ref() {
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
    let (name, args) = match exprs.split_first() {
        Some(split) => split,
        None => return Ok(Expr::List(vec![])),
    };

    if let Some(ret) = eval_list_builtin(name, args, env) {
        return ret;
    }

    let name = eval(name, env)?;

    let f = match &name {
        Expr::Function(f) => f,
        _ => return Err(EvalError::InvalidFunctionName(name.to_string())),
    };

    if args.len() != 2 {
        return Err(EvalError::InvalidArgumentCount);
    }

    let a = eval(&args[0], env)?;
    let b = eval(&args[1], env)?;

    (f.0)(&a, &b)
}

fn eval_list_builtin(name: &Expr, args: &[Expr], env: &Env) -> Option<Result<Expr, EvalError>> {
    match name {
        Expr::Symbol(s) if s.as_ref() == "def!" => Some(eval_builtin_def(args, env)),
        Expr::Symbol(s) if s.as_ref() == "let*" => Some(eval_builtin_let(args, env)),
        _ => None,
    }
}

fn eval_builtin_def(args: &[Expr], env: &Env) -> Result<Expr, EvalError> {
    let [key, val] = match args {
        [k, v] => [k, v],
        _ => return Err(EvalError::InvalidArgumentCount),
    };

    let key = match key.as_ref() {
        Expr::Symbol(s) => s,
        k => return Err(EvalError::InvalidVariableName(k.to_string())),
    };

    let val = eval(val, env)?;
    env.set(key, val.clone());

    Ok(val)
}

fn eval_builtin_let(args: &[Expr], env: &Env) -> Result<Expr, EvalError> {
    let [vars, expr] = match args {
        [v, e] => [v, e],
        _ => return Err(EvalError::InvalidArgumentCount),
    };

    let vars = match vars {
        Expr::List(l) if l.len() % 2 == 0 => l,
        Expr::Vector(l) if l.len() % 2 == 0 => l,
        _ => return Err(EvalError::InvalidLetVariables),
    };

    let let_env = Environment::with_parent(Rc::clone(env));

    for c in vars.chunks_exact(2) {
        let name = &c[0];
        let val = &c[1];

        let name = match name {
            Expr::Symbol(s) => s,
            _ => return Err(EvalError::InvalidLetVariables),
        };

        let val = eval(val, &let_env)?;
        let_env.set(name, val);
    }

    eval(expr, &let_env)
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
