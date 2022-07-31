use std::rc::Rc;

use crate::{
    ast::Expr,
    environment::{Env, Environment},
};

use self::builtins::eval_list_builtin;

mod builtins;

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("invalid function name: {0}")]
    InvalidFunctionName(String),
    #[error("invalid number of arguments")]
    InvalidArgumentCount,
    #[error("invalid function: {0}")]
    InvalidFunction(String),
    #[error("invalid function arguments: {0:?}")]
    InvalidArgumentTypes(Vec<String>),
    #[error("'{0}' not found")]
    UnknownSymbol(Rc<str>),
    #[error("invalid variable name: {0}")]
    InvalidVariableName(String),
    #[error("invalid variables for let*")]
    InvalidLetVariables,
}

pub type EvalResult<T> = std::result::Result<T, EvalError>;

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

    if args.len() != f.bindings.len() {
        return Err(EvalError::InvalidArgumentCount);
    }

    let args = args
        .iter()
        .map(|e| eval(e, env))
        .collect::<EvalResult<Vec<_>>>()?;

    let args_env = Environment::with_parent(f.closure.clone());

    for (binding, arg) in f.bindings.iter().zip(args) {
        args_env.set(binding, arg);
    }

    eval(&f.expr, &args_env)
}
