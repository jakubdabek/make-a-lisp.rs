use super::prelude::*;

pub(super) fn eval_is_map(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(expr, Expr::Map(_))))
}
