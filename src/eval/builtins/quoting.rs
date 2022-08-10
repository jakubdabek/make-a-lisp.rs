use super::prelude::*;

pub(super) fn eval_quote(args: &[Expr], _env: &Env) -> EvalResult<Expr> {
    args_n(args).map(|[arg]| arg.clone())
}

pub(super) fn eval_quasiquote(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_quasiquote_expand(args, env)?;
    eval::eval(&expr, env)
}

pub(super) fn eval_quasiquote_expand(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [arg] = args_n(args)?;

    match arg {
        Expr::List(l) => eval_quasiquote_list(l, env),
        Expr::Vector(v) => eval_quasiquote_vec(v, env),
        Expr::Symbol(_) | Expr::Map(_) => Ok(Expr::List(vec![
            Expr::BuiltinFunction("quote"),
            arg.clone(),
        ])),
        _ => Ok(arg.clone()),
    }
}

fn eval_quasiquote_vec(vec: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_quasiquote_list_like(vec, env).map(|l| Expr::List(vec![Expr::BuiltinFunction("vec"), l]))
}

fn eval_quasiquote_list(list: &[Expr], env: &Env) -> EvalResult<Expr> {
    match list {
        [name, args @ ..] if name.as_func_name() == Some("unquote") => {
            args_n(args).map(|[e]| e.clone())
        }
        _ => eval_quasiquote_list_like(list, env),
    }
}

fn eval_quasiquote_list_like(list: &[Expr], env: &Env) -> EvalResult<Expr> {
    list.iter().try_rfold(Expr::List(vec![]), |acc, elem| {
        let res = match elem.as_list_like() {
            Some([name, args @ ..]) if name.as_func_name() == Some("unquote") => {
                let [expr] = args_n(args)?;
                Expr::List(vec![Expr::BuiltinFunction("cons"), expr.clone(), acc])
            }
            Some([name, args @ ..]) if name.as_func_name() == Some("splice-unquote") => {
                let [expr] = args_n(args)?;
                Expr::List(vec![Expr::BuiltinFunction("concat"), expr.clone(), acc])
            }
            _ => {
                let expr = eval_quasiquote_expand(&[elem.clone()], env)?;
                Expr::List(vec![Expr::BuiltinFunction("cons"), expr, acc])
            }
        };
        Ok(res)
    })
}
