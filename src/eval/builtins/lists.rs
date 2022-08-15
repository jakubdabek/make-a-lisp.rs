use itertools::Itertools;

use super::prelude::*;

pub(super) fn eval_list(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env).map(Expr::List)
}

pub(super) fn eval_vec(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let list = into_list_like(arg)?;
    Ok(Expr::Vector(list))
}

pub(super) fn eval_vector(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env).map(Expr::Vector)
}

pub(super) fn eval_is_list(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(arg, Expr::List(_))))
}

pub(super) fn eval_is_vector(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(arg, Expr::Vector(_))))
}

pub(super) fn eval_is_sequential(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(arg, Expr::List(_) | Expr::Vector(_))))
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
    let tail = into_list_like(tail)?;
    Ok(Expr::List(std::iter::once(head).chain(tail).collect()))
}

pub(super) fn eval_concat(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;

    let list = args
        .into_iter()
        .map(into_list_like)
        .flatten_ok()
        .collect::<EvalResult<Vec<_>>>()?;

    Ok(Expr::List(list))
}

pub(super) fn eval_first(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let list = eval_1(args, env)?;

    if let Expr::Nil = list {
        return Ok(Expr::Nil);
    }

    let list = into_list_like(list)?;
    Ok(list.into_iter().next().unwrap_or(Expr::Nil))
}

pub(super) fn eval_rest(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let list = eval_1(args, env)?;

    if let Expr::Nil = list {
        return Ok(Expr::List(vec![]));
    }

    let mut list = into_list_like(list)?;
    if !list.is_empty() {
        list.remove(0);
    }

    Ok(Expr::List(list))
}

pub(super) fn eval_nth(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (list, idx) = eval_2(args, env)?;

    let idx = as_type(&idx, Expr::as_int)?;
    let list = into_list_like(list)?;
    let len = list.len();

    idx.try_into()
        .ok()
        .and_then(|idx| list.into_iter().nth(idx))
        .ok_or_else(|| EvalError::Exception(format!("index {idx} out of range for len {}", len)))
}
