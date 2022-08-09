use std::rc::Rc;

use crate::{
    ast::{display::Join, Expr, Function},
    environment::{Env, Environment},
};

use super::{
    eval, EvalError, EvalResult,
    Thunk::{self, Evaluated, Unevaluated},
};

const ARITHMETIC_BUILTINS: &[&str] = &["+", "-", "*", "/"];

const COMPARISON_BUILTINS: &[&str] = &["<", ">", ">=", "<="];

pub(super) fn eval_list_builtin(
    name: &Expr,
    args: &[Expr],
    env: &Env,
) -> Option<EvalResult<Thunk>> {
    let name: &str = name.as_symbol()?.as_ref();

    if let Some(thunk_result) = eval_thunk_builtin(name, args, env) {
        return Some(thunk_result);
    }

    let res = match name {
        "def!" => eval_def(args, env),
        "fn*" => eval_fn(args, env),
        "=" => eval_eq(args, env),
        "list" => eval_args(args, env).map(Expr::List),
        "list?" => Ok(Expr::Bool(matches!(args, [Expr::List(_)]))),
        "empty?" => args_n(args)
            .and_then(|[arg]| eval(arg, env))
            .map(|l| l.as_list_like().map(|l| l.is_empty()).unwrap_or(false))
            .map(Expr::Bool),
        "count" => args_n(args)
            .and_then(|[arg]| eval(arg, env))
            .map(|l| l.as_list_like().map(|l| l.len() as i64).unwrap_or(0))
            .map(Expr::Int),
        "pr-str" => eval_args(args, env)
            .map(|args| format!("{:#}", Join(&args, " ")))
            .map(Expr::String),
        "str" => eval_args(args, env)
            .map(|args| format!("{}", Join(&args, "")))
            .map(Expr::String),
        "prn" => eval_args(args, env)
            .map(|args| println!("{:#}", Join(&args, " ")))
            .map(|_| Expr::Nil),
        "println" => eval_args(args, env)
            .map(|args| println!("{}", Join(&args, " ")))
            .map(|_| Expr::Nil),
        n if ARITHMETIC_BUILTINS.contains(&n) => eval_arithmetic(n, args, env),
        n if COMPARISON_BUILTINS.contains(&n) => eval_cmp(n, args, env),
        _ => return None,
    };

    Some(res.map(Evaluated))
}

fn eval_thunk_builtin(name: &str, args: &[Expr], env: &Env) -> Option<EvalResult<Thunk>> {
    let res = match name {
        "let*" => eval_let(args, env),
        "do" => eval_do(args, env),
        "if" => eval_if(args, env),
        _ => return None,
    };

    Some(res)
}

fn eval_args(args: &[Expr], env: &Env) -> EvalResult<Vec<Expr>> {
    args.iter().map(|arg| super::eval(arg, env)).collect()
}

fn args_n<const N: usize>(args: &[Expr]) -> EvalResult<&[Expr; N]> {
    args.try_into().map_err(|_| EvalError::InvalidArgumentCount)
}

fn eval_2(args: &[Expr], env: &Env) -> EvalResult<(Expr, Expr)> {
    let [a, b] = args_n(args)?;
    let [a, b] = [a, b].map(|e| super::eval(e, env));

    Ok((a?, b?))
}

fn eval_number_args(args: &[Expr], env: &Env) -> EvalResult<(i64, i64)> {
    let (a, b) = eval_2(args, env)?;

    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Ok((a, b)),
        (a, b) => Err(EvalError::InvalidArgumentTypes(vec![
            a.to_string(),
            b.to_string(),
        ])),
    }
}

fn eval_arithmetic(s: &str, args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_number_args(args, env)?;
    let res = match s {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" => a / b,
        _ => unreachable!(),
    };

    Ok(Expr::Int(res))
}

fn eval_cmp(s: &str, args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_number_args(args, env)?;
    let res = match s {
        "<" => a < b,
        ">" => a > b,
        "<=" => a <= b,
        ">=" => a >= b,
        _ => unreachable!(),
    };

    Ok(Expr::Bool(res))
}

fn eval_do(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let thunk = match args {
        [] => Evaluated(Expr::Nil),
        [args @ .., last] => {
            let _ = args
                .iter()
                .try_fold(Expr::Nil, |_, arg| super::eval(arg, env))?;
            Unevaluated(last.clone(), env.clone())
        }
    };

    Ok(thunk)
}

fn eval_fn(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [bindings, expr] = args_n(args)?;
    let bindings = match bindings {
        Expr::List(b) | Expr::Vector(b) => b,
        _ => return Err(EvalError::InvalidArgumentTypes(vec![bindings.to_string()])),
    };

    let bindings = bindings
        .iter()
        .map(|b| {
            b.as_symbol()
                .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![b.to_string()]))
        })
        .collect::<EvalResult<Vec<_>>>()?;

    let bindings = bindings.into_iter().map(<str>::to_owned).collect();
    let expr = Rc::new(expr.clone());

    Ok(Expr::Function(Function {
        bindings,
        expr,
        closure: env.clone(),
    }))
}

fn eval_if(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let [cond, success, failure] = match args_n(args) {
        Ok([c, s, f]) => [c, s, f],
        Err(EvalError::InvalidArgumentCount) => {
            let [c, s] = args_n(args)?;
            [c, s, &Expr::NIL]
        }
        Err(_) => unreachable!(),
    };

    let cond = eval(cond, env)?;

    match cond {
        Expr::Nil | Expr::Bool(false) => Ok(Unevaluated(failure.clone(), env.clone())),
        _ => Ok(Unevaluated(success.clone(), env.clone())),
    }
}

fn eval_eq(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_2(args, env)?;
    Ok(Expr::Bool(a.lenient_eq(&b)))
}

fn eval_def(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [key, val] = args_n(args)?;
    let key = key
        .as_symbol()
        .ok_or_else(|| EvalError::InvalidVariableName(key.to_string()))?;

    let val = super::eval(val, env)?;
    env.set(key, val.clone());

    Ok(val)
}

fn eval_let(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let [vars, expr] = args_n(args)?;

    let vars = match vars {
        Expr::List(l) if l.len() % 2 == 0 => l,
        Expr::Vector(l) if l.len() % 2 == 0 => l,
        _ => return Err(EvalError::InvalidLetVariables),
    };

    let let_env = Environment::with_parent(env.clone());

    for c in vars.chunks_exact(2) {
        let name = &c[0];
        let val = &c[1];

        let name = name.as_symbol().ok_or(EvalError::InvalidLetVariables)?;

        let val = super::eval(val, &let_env)?;
        let_env.set(name, val);
    }

    Ok(Unevaluated(expr.clone(), let_env))
}
