use std::rc::Rc;

use super::prelude::*;

pub(super) fn eval_meta(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    match expr {
        Expr::WithMeta { meta, .. } => Ok(Expr::clone(&*meta)),
        _ => Ok(Expr::Nil),
    }
}

pub(super) fn eval_with_meta(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (expr, meta) = eval_2(args, env)?;
    match expr {
        Expr::WithMeta { expr, .. } => Ok(Expr::WithMeta {
            expr,
            meta: Rc::new(meta),
        }),
        Expr::List(_)
        | Expr::Vector(_)
        | Expr::Map(_)
        | Expr::Function(_)
        | Expr::BuiltinFunction(_) => Ok(Expr::WithMeta {
            expr: Rc::new(expr),
            meta: Rc::new(meta),
        }),
        _ => Err(EvalError::InvalidArgumentTypes(vec![expr.to_string()])),
    }
}
