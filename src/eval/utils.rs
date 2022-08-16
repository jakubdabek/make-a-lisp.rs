use crate::{ast::Expr, environment::Env};

use super::{EvalError, EvalResult};

pub(super) fn eval_args(args: &[Expr], env: &Env) -> EvalResult<Vec<Expr>> {
    args.iter().map(|arg| super::eval(arg, env)).collect()
}

#[cfg(feature = "nightly")]
pub(super) fn eval_n<const N: usize>(args: &[Expr], env: &Env) -> EvalResult<[Expr; N]> {
    let args = args_n(args)?;
    args.try_map(|arg| super::eval(&arg, env))
}

pub(super) fn args_n<const N: usize>(args: &[Expr]) -> EvalResult<&[Expr; N]> {
    args.try_into().map_err(|_| EvalError::InvalidArgumentCount)
}

pub(super) fn eval_1(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [a] = args_n(args)?;
    super::eval(a, env)
}

pub(super) fn eval_2(args: &[Expr], env: &Env) -> EvalResult<(Expr, Expr)> {
    let [a, b] = args_n(args)?;
    let [a, b] = [a, b].map(|e| super::eval(e, env));

    Ok((a?, b?))
}

pub(super) fn into_list_like(arg: Expr) -> EvalResult<Vec<Expr>> {
    into_type(arg, Expr::into_list_like)
}

pub(super) fn as_type<'a, T: 'a>(
    arg: &'a Expr,
    op: impl FnOnce(&'a Expr) -> Option<T>,
) -> EvalResult<T> {
    op(arg.as_no_meta()).ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))
}

pub(super) fn into_type<T>(arg: Expr, op: impl FnOnce(Expr) -> Result<T, Expr>) -> EvalResult<T> {
    op(arg.into_no_meta()).map_err(|arg| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))
}

pub(super) mod macros {
    macro_rules! as_type_fn {
        ( $variant:path ) => {
            |e| match e {
                $variant(e) => Some(e),
                _ => None,
            }
        };
    }
    pub(crate) use as_type_fn;

    macro_rules! as_type {
        ( $e:expr => $variant:path ) => {
            as_type($e, super::macros::as_type_fn!($variant))
        };
    }
    pub(crate) use as_type;

    macro_rules! into_type_fn {
        ( $variant:path ) => {
            |e| match e {
                $variant(e) => Ok(e),
                _ => Err(e),
            }
        };
    }
    pub(crate) use into_type_fn;

    macro_rules! into_type {
        ( $e:expr => $variant:path ) => {
            into_type($e, super::macros::into_type_fn!($variant))
        };
    }
    pub(crate) use into_type;

    macro_rules! is_type {
        ($args:expr, $env:expr, $($pat:tt)+) => {{
            let expr = eval_1($args, $env)?;
            Ok(Expr::Bool(matches!(expr.as_no_meta(), $($pat)+)))
        }};
    }
    pub(crate) use is_type;
}

pub(super) fn eval_number_args(args: &[Expr], env: &Env) -> EvalResult<(i64, i64)> {
    let (a, b) = eval_2(args, env)?;

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Ok((a, b)),
        (a, b) => Err(EvalError::InvalidArgumentTypes(vec![
            a.to_string(),
            b.to_string(),
        ])),
    }
}
