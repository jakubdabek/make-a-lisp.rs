use std::{io, rc::Rc};

use crate::{
    ast::{Expr, Function, MapKey},
    environment::{Env, Environment},
    parser::ParseError,
};

use self::builtins::eval_list_builtin;

pub mod builtins;
mod utils;

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
    Unevaluated(Rc<Expr>, Env),
}

use fnv::FnvHashMap;
use Thunk::{Evaluated, Unevaluated};

pub fn eval(expr: &Expr, env: &Env) -> EvalResult<Expr> {
    eval_maybe_macro(expr, env, true)
}

fn is_macro(expr: &Expr, env: &Env) -> bool {
    let l = match expr {
        Expr::List(l) => l,
        _ => return false,
    };

    let name = match l.as_slice() {
        [name, ..] => name,
        _ => return false,
    };

    let name = match name.as_func_name() {
        Some(name) => name,
        None => return false,
    };

    let f = match env.get(name) {
        Some(f) => f,
        None => return false,
    };

    matches!(f, Expr::Function(Function { is_macro: true, .. }))
}

fn eval_maybe_macro(expr: &Expr, env: &Env, expand_macros: bool) -> EvalResult<Expr> {
    let mut expr_owner;
    let mut expr = &*expr;
    let mut env_owner;
    let mut env = &*env;
    let mut last_macro = false;
    loop {
        // eprintln!("last_macro = {last_macro}, expr = {}", expr);
        // eprintln!("{:#?}", env);
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
                        let (e, new_env) = match &*e {
                            Expr::MacroExpand(e) if expand_macros => {
                                // top-level macro, needs to expand and then evaluate in the current env
                                let e = eval_maybe_macro(e, &new_env, false)?;
                                last_macro = true;
                                (Rc::new(e), env.clone())
                            }
                            Expr::MacroExpand(e) => {
                                last_macro = true;
                                (Rc::clone(e), new_env)
                            }
                            _ => (e, new_env),
                        };
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
            Expr::Map(m) => eval_map_literal(m, env),
            expr => Ok(expr.clone()),
        };

        let evaluated = evaluated?;
        if last_macro && (expand_macros || is_macro(&evaluated, env)) {
            expr_owner = Rc::new(evaluated);
            expr = &expr_owner;
            last_macro = false;
            continue;
        }

        break Ok(evaluated);
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

    let mut args = if f.is_macro {
        args.to_vec()
    } else {
        args.iter()
            .map(|e| eval(e, env))
            .collect::<EvalResult<Vec<_>>>()?
    };

    let args_env = Environment::with_parent(f.closure.clone());

    if let Some(varargs_name) = &f.varargs {
        let varargs = args.split_off(f.bindings.len());
        args_env.set(varargs_name, Expr::List(varargs));
    }

    for (binding, arg) in f.bindings.iter().zip(args) {
        args_env.set(binding, arg);
    }

    if f.is_macro {
        Ok(Unevaluated(
            Rc::new(Expr::MacroExpand(Rc::clone(&f.expr))),
            args_env,
        ))
    } else {
        Ok(Unevaluated(Rc::clone(&f.expr), args_env))
    }
}

fn eval_map_literal(map: &FnvHashMap<MapKey, Expr>, env: &Env) -> EvalResult<Expr> {
    let map = map
        .iter()
        .map(|(k, v)| Ok((k.clone(), eval(v, env)?)))
        .collect::<EvalResult<_>>()?;
    Ok(Expr::Map(Rc::new(map)))
}
