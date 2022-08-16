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
    is_type!(args, env, Expr::List(_))
}

pub(super) fn eval_is_vector(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Vector(_))
}

pub(super) fn eval_is_sequential(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::List(_) | Expr::Vector(_))
}

pub(super) fn eval_is_empty(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Bool(
        arg.as_no_meta()
            .as_list_like()
            .map(|l| l.is_empty())
            .unwrap_or(false),
    ))
}

pub(super) fn eval_count(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    Ok(Expr::Int(
        arg.as_no_meta()
            .as_list_like()
            .map(|l| l.len() as i64)
            .unwrap_or(0),
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
        .ok_or_else(|| {
            EvalError::Exception(Expr::String(format!(
                "index {idx} out of range for len {}",
                len
            )))
        })
}

pub(super) fn eval_conj(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;
    let (seq, args) = args.split_first().ok_or(EvalError::InvalidArgumentCount)?;

    match seq.as_no_meta() {
        Expr::List(l) => Ok(Expr::List(
            args.iter()
                .cloned()
                .rev()
                .chain(l.iter().cloned())
                .collect(),
        )),
        Expr::Vector(v) => Ok(Expr::Vector(
            v.iter().cloned().chain(args.iter().cloned()).collect(),
        )),
        seq => Err(EvalError::InvalidArgumentTypes(vec![seq.to_string()])),
    }
}

pub(super) fn eval_seq(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    match arg.into_no_meta() {
        Expr::Nil => Ok(Expr::Nil),
        Expr::List(l) if l.is_empty() => Ok(Expr::Nil),
        l @ Expr::List(_) => Ok(l),
        Expr::Vector(v) if v.is_empty() => Ok(Expr::Nil),
        Expr::Vector(v) => Ok(Expr::List(v)),
        Expr::String(s) if s.is_empty() => Ok(Expr::Nil),
        Expr::String(s) => Ok(Expr::List(
            s.chars().map(|c| c.to_string()).map(Expr::String).collect(),
        )),
        arg => Err(EvalError::InvalidArgumentTypes(vec![arg.to_string()])),
    }
}
