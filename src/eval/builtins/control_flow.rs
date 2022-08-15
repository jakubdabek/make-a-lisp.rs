use std::rc::Rc;

use crate::{
    ast::Function,
    eval::Thunk::{self, Evaluated, Unevaluated},
};

use super::{prelude::*, quoting::make_quote};

pub(super) fn eval_do(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let thunk = match args {
        [] => Evaluated(Expr::Nil),
        [args @ .., last] => {
            let _ = args
                .iter()
                .try_fold(Expr::Nil, |_, arg| super::eval(arg, env))?;
            Unevaluated(Rc::new(last.clone()), env.clone())
        }
    };

    Ok(thunk)
}

pub(super) fn eval_fn(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [bindings, expr] = args_n(args)?;
    let bindings = as_type(bindings, Expr::as_list_like)?;

    let mut bindings = bindings
        .iter()
        .map(|b| as_type(b, Expr::as_symbol))
        .collect::<EvalResult<Vec<_>>>()?;

    let varargs = bindings.iter().filter(|&&b| b == "&").count();
    let varargs = match varargs {
        0 => None,
        1 => {
            let vararg_name = bindings.pop().unwrap();
            let _sigil = bindings
                .pop()
                .filter(|&b| b == "&")
                .ok_or(EvalError::InvalidVarargs)?;
            Some(vararg_name.to_owned())
        }
        _ => return Err(EvalError::InvalidVarargs),
    };

    let bindings = bindings.into_iter().map(<str>::to_owned).collect();
    let expr = Rc::new(expr.clone());

    Ok(Expr::Function(Function {
        bindings,
        varargs,
        expr,
        closure: env.clone(),
        is_macro: false,
    }))
}

pub(super) fn eval_if(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
    let [cond, success, failure] = match args_n(args) {
        Ok([c, s, f]) => [c, s, f],
        Err(EvalError::InvalidArgumentCount) => {
            let [c, s] = args_n(args)?;
            [c, s, &Expr::NIL]
        }
        Err(_) => unreachable!(),
    };

    let cond = eval::eval(cond, env)?;

    match cond {
        Expr::Nil | Expr::Bool(false) => Ok(Unevaluated(Rc::new(failure.clone()), env.clone())),
        _ => Ok(Unevaluated(Rc::new(success.clone()), env.clone())),
    }
}

fn eval_def_inner(
    args: &[Expr],
    env: &Env,
    modify: impl FnOnce(Expr) -> EvalResult<Expr>,
) -> EvalResult<Expr> {
    let [key, val] = args_n(args)?;
    let key = key
        .as_symbol()
        .ok_or_else(|| EvalError::InvalidVariableName(key.to_string()))?;

    let val = super::eval(val, env)?;
    let val = modify(val)?;
    env.set(key, val.clone());

    Ok(val)
}

pub(super) fn eval_def(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_def_inner(args, env, Ok)
}

pub(super) fn eval_def_macro(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    eval_def_inner(args, env, |e| {
        into_type!(e => Expr::Function).map(|f| {
            Expr::Function(Function {
                is_macro: true,
                ..f
            })
        })
    })
}

pub(super) fn eval_let(args: &[Expr], env: &Env) -> EvalResult<Thunk> {
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

    Ok(Unevaluated(Rc::new(expr.clone()), let_env))
}

pub(super) fn eval_try(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (expr, catch) = match args_n(args) {
        Ok([expr, catch]) => (expr, catch),
        Err(_) => {
            let [expr] = args_n(args)?;
            return super::eval(expr, env);
        }
    };
    let catch = as_type!(catch => Expr::List)?;
    let invalid = || EvalError::InvalidCatchBlock;
    let [catch, catch_var, catch_expr] = args_n(catch).map_err(|_| invalid())?;
    let catch = as_type(catch, Expr::as_symbol).map_err(|_| invalid())?;
    let catch_var = as_type(catch_var, Expr::as_symbol).map_err(|_| invalid())?;
    if catch != "catch*" {
        return Err(invalid());
    }

    let exc = match super::eval(expr, env) {
        Err(EvalError::Exception(exc)) => exc,
        res => return res,
    };

    let catch_func = Function {
        bindings: vec![catch_var.to_owned()],
        varargs: None,
        expr: Rc::new(catch_expr.clone()),
        closure: env.clone(),
        is_macro: false,
    };
    super::eval(
        &Expr::List(vec![Expr::Function(catch_func), make_quote(exc)]),
        env,
    )
}

pub(super) fn eval_throw(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    Err(EvalError::Exception(expr))
}
