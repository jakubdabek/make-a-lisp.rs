use std::{io, rc::Rc};

use crate::{
    ast::Expr,
    environment::{Env, Environment},
    parser::ParseError,
};

use self::builtins::eval_list_builtin;

pub mod builtins;

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
    #[error("invalid vararg arguments")]
    InvalidVarargs,
    #[error("'{0}' not found")]
    UnknownSymbol(Rc<str>),
    #[error("invalid variable name: {0}")]
    InvalidVariableName(String),
    #[error("invalid variables for let*")]
    InvalidLetVariables,
    #[error("exception occurred: {0}")]
    Exception(String),
    #[error("parsing error: {0}")]
    ParseError(#[from] ParseError),
    #[error("IO error: {0}")]
    IOError(#[from] io::Error),
}

pub type EvalResult<T> = std::result::Result<T, EvalError>;

#[derive(Debug)]
enum Thunk {
    Evaluated(Expr),
    Unevaluated(Expr, Env),
}

use Thunk::{Evaluated, Unevaluated};

pub fn eval(expr: &Expr, env: &Env) -> EvalResult<Expr> {
    let mut expr_owner;
    let mut expr = &*expr;
    let mut env_owner;
    let mut env = &*env;
    loop {
        let evaluated = match expr {
            Expr::Symbol(sym) => match env.get(&**sym) {
                Some(f) => Ok(f),
                None => Err(EvalError::UnknownSymbol(sym.clone())),
            },
            Expr::List(v) => {
                let thunk = eval_list(v, env)?;
                match thunk {
                    Evaluated(e) => Ok(e),
                    Unevaluated(e, new_env) => {
                        expr_owner = e;
                        expr = &expr_owner;
                        env_owner = new_env;
                        env = &env_owner;
                        continue;
                    }
                }
            }
            Expr::Vector(v) => Ok(Expr::Vector(
                v.iter().map(|e| eval(e, env)).collect::<EvalResult<_>>()?,
            )),
            Expr::Map(m) => Ok(Expr::Map(
                m.iter().map(|e| eval(e, env)).collect::<EvalResult<_>>()?,
            )),
            expr => Ok(expr.clone()),
        };

        break evaluated;
    }
}

fn eval_list(exprs: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let (name, args) = match exprs.split_first() {
        Some(split) => split,
        None => return Ok(Evaluated(Expr::List(vec![]))),
    };

    if let Some(ret) = eval_list_builtin(name, args, env) {
        return ret;
    }

    let name = eval(name, env)?;

    let f = match &name {
        Expr::Function(f) => f,
        _ => return Err(EvalError::InvalidFunctionName(name.to_string())),
    };

    use std::cmp::Ordering::*;
    match args.len().cmp(&f.bindings.len()) {
        Equal if f.varargs.is_none() => {}
        Greater | Equal if f.varargs.is_some() => {}
        _ => return Err(EvalError::InvalidArgumentCount),
    }

    let mut args = args
        .iter()
        .map(|e| eval(e, env))
        .collect::<EvalResult<Vec<_>>>()?;

    let args_env = Environment::with_parent(f.closure.clone());

    if let Some(varargs_name) = &f.varargs {
        let varargs = args.split_off(f.bindings.len());
        args_env.set(varargs_name, Expr::List(varargs));
    }

    for (binding, arg) in f.bindings.iter().zip(args) {
        args_env.set(binding, arg);
    }

    Ok(Unevaluated(Expr::clone(&f.expr), args_env))
}
