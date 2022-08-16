use crate::{
    ast::{Function, Keyword},
    parser,
};

use super::prelude::*;

pub(super) fn eval_is_false(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Bool(false))
}

pub(super) fn eval_is_true(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Bool(true))
}

pub(super) fn eval_is_nil(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Nil)
}

pub(super) fn eval_is_symbol(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Symbol(_))
}

pub(super) fn eval_is_string(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::String(_))
}

pub(super) fn eval_is_number(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Int(_))
}

pub(super) fn eval_is_fn(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(
        args,
        env,
        Expr::Function(Function {
            is_macro: false,
            ..
        }) | Expr::BuiltinFunction(_)
    )
}

pub(super) fn eval_is_macro(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Function(Function { is_macro: true, .. }))
}

pub(super) fn eval_symbol(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    let s = as_type(&expr, Expr::as_string)?;
    let s = parser::parse(s)?;

    match s {
        Expr::Symbol(_) => Ok(s),
        _ => Err(EvalError::InvalidArgumentTypes(vec![s.to_string()])),
    }
}

pub(super) fn eval_is_keyword(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Keyword(_))
}

pub(super) fn eval_keyword(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    if matches!(expr, Expr::Keyword(_)) {
        return Ok(expr);
    }
    let s = as_type(&expr, Expr::as_string)?;
    let s = parser::parse(s)?;

    match s {
        Expr::Keyword(_) => Ok(s),
        Expr::Symbol(s) => Ok(Expr::Keyword(Keyword::new(&*s))),
        _ => Err(EvalError::InvalidArgumentTypes(vec![s.to_string()])),
    }
}
