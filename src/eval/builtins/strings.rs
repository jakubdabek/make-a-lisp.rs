use crate::{ast::display::Join, parser, repl};

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
    let arg = as_type(&arg, Expr::as_string)?;

    Ok(parser::parse(arg)?)
}

pub(super) fn eval_slurp(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let arg = as_type(&arg, Expr::as_string)?;

    let content = std::fs::read_to_string(arg)?;
    Ok(Expr::String(content))
}

pub(super) fn eval_readline(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let pr = eval_1(args, env)?;
    let pr = as_type(&pr, Expr::as_string)?;

    match repl::read(Some(&format!("\n{}", pr))) {
        Ok(s) if s.is_empty() => Ok(Expr::Nil),
        Ok(s) => Ok(Expr::String(s)),
        Err(repl::Error::IO(err)) => Err(EvalError::IOError(err)),
        Err(_) => unreachable!("repl::read doesn't have other error conditions"),
    }
}
