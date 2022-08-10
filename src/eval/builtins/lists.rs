use itertools::Itertools;

use super::prelude::*;

pub(super) fn eval_list(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env).map(Expr::List)
}

pub(super) fn eval_vec(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let list = arg
        .into_list_like()
        .map_err(|arg| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))?;
    Ok(Expr::Vector(list))
}

pub(super) fn eval_is_list(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(arg, Expr::List(_))))
}

pub(super) fn eval_is_vector(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(arg, Expr::Vector(_))))
}

pub(super) fn eval_is_empty(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(
        arg.as_list_like().map(|l| l.is_empty()).unwrap_or(false),
    ))
}

pub(super) fn eval_count(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Int(
        arg.as_list_like().map(|l| l.len() as i64).unwrap_or(0),
    ))
}

pub(super) fn eval_cons(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (head, tail) = eval_2(args, env)?;
    let tail = tail
        .into_list_like()
        .map_err(|tail| EvalError::InvalidArgumentTypes(vec![tail.to_string()]))?;
    Ok(Expr::List(std::iter::once(head).chain(tail).collect()))
}

pub(super) fn eval_concat(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;

    let list = args
        .into_iter()
        .map(|arg| {
            arg.into_list_like()
                .map_err(|arg| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))
        })
        .flatten_ok()
        .collect::<EvalResult<Vec<_>>>()?;

    Ok(Expr::List(list))
}
