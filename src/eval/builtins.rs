use std::rc::Rc;

use crate::{
    ast::{display::Join, Expr, Function},
    environment::{Env, Environment},
    parser,
};

use super::{
    eval, EvalError, EvalResult,
    Thunk::{self, Evaluated, Unevaluated},
};

// const ARITHMETIC_BUILTINS: &[&str] = &["+", "-", "*", "/"];
// const COMPARISON_BUILTINS: &[&str] = &["<", ">", ">=", "<="];

pub(super) type BuiltinThunkFn = fn(&[Expr], &Env) -> EvalResult<Thunk>;
pub type BuiltinFn = fn(&[Expr], &Env) -> EvalResult<Expr>;

macro_rules! number_op {
    ( $func:ident ( $op:tt ) ) => {
        |args, env| $func(|a, b| a $op b, args, env)
    };
}

pub const BUILTINS: &[(&str, BuiltinFn)] = &[
    ("def!", eval_def),
    ("fn*", eval_fn),
    ("=", eval_eq),
    ("list", |args, env| eval_args(args, env).map(Expr::List)),
    ("list?", |args, env| {
        eval_1(args, env).map(|l| Expr::Bool(matches!(l, Expr::List(_))))
    }),
    ("empty?", |args, env| {
        eval_1(args, env)
            .map(|l| l.as_list_like().map(|l| l.is_empty()).unwrap_or(false))
            .map(Expr::Bool)
    }),
    ("count", |args, env| {
        eval_1(args, env)
            .map(|l| l.as_list_like().map(|l| l.len() as i64).unwrap_or(0))
            .map(Expr::Int)
    }),
    ("pr-str", |args, env| {
        eval_args(args, env)
            .map(|args| format!("{:#}", Join(&args, " ")))
            .map(Expr::String)
    }),
    ("str", |args, env| {
        eval_args(args, env)
            .map(|args| format!("{}", Join(&args, "")))
            .map(Expr::String)
    }),
    ("prn", |args, env| {
        eval_args(args, env)
            .map(|args| println!("{:#}", Join(&args, " ")))
            .map(|_| Expr::Nil)
    }),
    ("println", |args, env| {
        eval_args(args, env)
            .map(|args| println!("{}", Join(&args, " ")))
            .map(|_| Expr::Nil)
    }),
    ("slurp", eval_slurp),
    ("read-string", eval_read_string),
    ("eval", eval_eval),
    ("atom", |args, env| eval_1(args, env).map(Expr::atom)),
    ("atom?", |args, env| {
        eval_1(args, env).map(|e| Expr::Bool(matches!(e, Expr::Atom(_))))
    }),
    ("deref", eval_deref),
    ("reset!", eval_reset),
    ("swap!", eval_swap),
    ("+", number_op!(eval_arithmetic(+))),
    ("-", number_op!(eval_arithmetic(-))),
    ("*", number_op!(eval_arithmetic(*))),
    ("/", number_op!(eval_arithmetic(/))),
    (">", number_op!(eval_cmp(>))),
    ("<", number_op!(eval_cmp(<))),
    (">=", number_op!(eval_cmp(>=))),
    ("<=", number_op!(eval_cmp(<=))),
];

pub(super) const THUNK_BUILTINS: &[(&str, BuiltinThunkFn)] =
    &[("let*", eval_let), ("do", eval_do), ("if", eval_if)];

pub(super) fn eval_list_builtin(
    name: &Expr,
    args: &[Expr],
    env: &Env,
) -> Option<EvalResult<Thunk>> {
    let name: &str = name.as_symbol().or_else(|| name.as_builtin())?.as_ref();

    if let Some(thunk_result) = eval_thunk_builtin(name, args, env) {
        return Some(thunk_result);
    }

    BUILTINS
        .iter()
        .find(|&&(fname, _)| fname == name)
        .map(|(_, f)| f(args, env))
        .map(|res| res.map(Evaluated))
}

fn eval_thunk_builtin(name: &str, args: &[Expr], env: &Env) -> Option<EvalResult<Thunk>> {
    THUNK_BUILTINS
        .iter()
        .find(|&&(fname, _)| fname == name)
        .map(|(_, f)| f(args, env))
}

fn eval_args(args: &[Expr], env: &Env) -> EvalResult<Vec<Expr>> {
    args.iter().map(|arg| super::eval(arg, env)).collect()
}

#[cfg(feature = "nightly")]
fn eval_n<const N: usize>(args: &[Expr], env: &Env) -> EvalResult<[Expr; N]> {
    let args = args_n(args)?;
    args.try_map(|arg| super::eval(&arg, env))
}

fn args_n<const N: usize>(args: &[Expr]) -> EvalResult<&[Expr; N]> {
    args.try_into().map_err(|_| EvalError::InvalidArgumentCount)
}

fn eval_1(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let [a] = args_n(args)?;
    super::eval(a, env)
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

fn eval_read_string(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let arg = arg
        .as_string()
        .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))?;

    Ok(parser::parse(arg)?)
}

fn eval_slurp(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let arg = eval_1(args, env)?;
    let arg = arg
        .as_string()
        .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))?;

    let content = std::fs::read_to_string(arg)?;
    Ok(Expr::String(content))
}

fn eval_eval(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    let env = env.top_level_env();
    super::eval(&expr, env)
}

fn eval_deref(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    match expr {
        Expr::Atom(a) => Ok(a.borrow().clone()),
        _ => Err(EvalError::InvalidArgumentTypes(vec![expr.to_string()])),
    }
}

fn eval_reset(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (atom, expr) = eval_2(args, env)?;
    let atom = match atom {
        Expr::Atom(a) => a,
        _ => return Err(EvalError::InvalidArgumentTypes(vec![atom.to_string()])),
    };

    *atom.borrow_mut() = expr.clone();
    Ok(expr)
}

fn eval_swap(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let mut args = eval_args(args, env)?;
    let atom_ref = match &mut args[..] {
        [atom, func, ..] => {
            std::mem::swap(atom, func);
            let (atom, _func) = (func, atom);
            let atom_ref = match atom {
                Expr::Atom(a) => a.clone(),
                _ => return Err(EvalError::InvalidArgumentTypes(vec![atom.to_string()])),
            };
            *atom = atom_ref.borrow().clone();
            atom_ref
        }
        _ => return Err(EvalError::InvalidArgumentCount),
    };

    let value = eval(&Expr::List(args), env)?;
    *atom_ref.borrow_mut() = value.clone();
    Ok(value)
}

fn eval_arithmetic(op: impl FnOnce(i64, i64) -> i64, args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_number_args(args, env)?;
    let res = op(a, b);

    Ok(Expr::Int(res))
}

fn eval_cmp(op: impl FnOnce(i64, i64) -> bool, args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_number_args(args, env)?;
    let res = op(a, b);

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

    let mut bindings = bindings
        .iter()
        .map(|b| {
            b.as_symbol()
                .ok_or_else(|| EvalError::InvalidArgumentTypes(vec![b.to_string()]))
        })
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
