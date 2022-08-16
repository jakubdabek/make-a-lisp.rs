use super::prelude::*;

pub(super) fn eval_meta(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let _expr = eval_1(args, env)?;
    Ok(Expr::Nil)
}

pub(super) fn eval_with_meta(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (expr, _meta) = eval_2(args, env)?;
    Ok(expr)
}
