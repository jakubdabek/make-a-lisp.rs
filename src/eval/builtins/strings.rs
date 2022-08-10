use crate::{ast::display::Join, parser};

use super::prelude::*;

pub(super) fn eval_pr_str(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env)
        .map(|args| format!("{:#}", Join(&args, " ")))
        .map(Expr::String)
}

pub(super) fn eval_str(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env)
        .map(|args| format!("{}", Join(&args, "")))
        .map(Expr::String)
}

pub(super) fn eval_prn(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env)
        .map(|args| println!("{:#}", Join(&args, " ")))
        .map(|_| Expr::Nil)
}

pub(super) fn eval_println(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_args(args, env)
        .map(|args| println!("{}", Join(&args, " ")))
        .map(|_| Expr::Nil)
}

pub(super) fn eval_read_string(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let arg = arg
        .as_string()
        .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))?;

    Ok(parser::parse(arg)?)
}

pub(super) fn eval_slurp(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let arg = arg
        .as_string()
        .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))?;

    let content = std::fs::read_to_string(arg)?;
    Ok(Expr::String(content))
}
