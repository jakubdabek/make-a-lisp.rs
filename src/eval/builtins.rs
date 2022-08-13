use crate::{ast::Expr, environment::Env};

use super::{
    eval, EvalError, EvalResult,
    Thunk::{self, Evaluated},
};

mod atoms;
mod control_flow;
mod lists;
mod quoting;
mod strings;

mod prelude {
    pub(super) use super::{
        args_n, as_type, eval_1, eval_2, eval_args, into_list_like, into_type, macros::*,
    };
    pub(super) use crate::{
        ast::Expr,
        environment::{Env, Environment},
        eval::{self, EvalError, EvalResult},
    };
}

use self::{atoms::*, control_flow::*, lists::*, quoting::*, strings::*};

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
    ("defmacro!", eval_def_macro),
    ("fn*", eval_fn),
    ("=", eval_eq),
    // lists
    ("list", eval_list),
    ("list?", eval_is_list),
    ("empty?", eval_is_empty),
    ("count", eval_count),
    ("cons", eval_cons),
    ("first", eval_first),
    ("rest", eval_rest),
    ("nth", eval_nth),
    ("concat", eval_concat),
    ("vec", eval_vec),
    ("vector?", eval_is_vector),
    // strings
    ("pr-str", eval_pr_str),
    ("str", eval_str),
    ("prn", eval_prn),
    ("println", eval_println),
    ("slurp", eval_slurp),
    ("read-string", eval_read_string),
    // quoting
    ("eval", eval_eval),
    ("eval*", eval_eval_local),
    ("quote", eval_quote),
    // ("unquote", eval_unquote),
    ("quasiquoteexpand", eval_quasiquote_expand),
    // ("splice-unquote", eval_splice_unquote),
    ("macroexpand", eval_macro_expand),
    // atoms
    ("atom", eval_atom),
    ("atom?", eval_is_atom),
    ("deref", eval_deref),
    ("reset!", eval_reset),
    ("swap!", eval_swap),
    // numbers
    ("+", number_op!(eval_arithmetic(+))),
    ("-", number_op!(eval_arithmetic(-))),
    ("*", number_op!(eval_arithmetic(*))),
    ("/", number_op!(eval_arithmetic(/))),
    (">", number_op!(eval_cmp(>))),
    ("<", number_op!(eval_cmp(<))),
    (">=", number_op!(eval_cmp(>=))),
    ("<=", number_op!(eval_cmp(<=))),
];

pub(super) const THUNK_BUILTINS: &[(&str, BuiltinThunkFn)] = &[
    ("let*", eval_let),
    ("do", eval_do),
    ("if", eval_if),
    ("quasiquote", eval_quasiquote),
];

pub(super) fn eval_list_builtin(
    name: &Expr,
    args: &[Expr],
    env: &Env,
) -> Option<EvalResult<Thunk>> {
    let name: &str = name.as_func_name()?.as_ref();

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

fn into_list_like(arg: Expr) -> EvalResult<Vec<Expr>> {
    into_type(arg, Expr::into_list_like)
}

fn as_type<'a, T: 'a>(arg: &'a Expr, op: impl FnOnce(&'a Expr) -> Option<T>) -> EvalResult<T> {
    op(arg).ok_or_else(|| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))
}

fn into_type<T>(arg: Expr, op: impl FnOnce(Expr) -> Result<T, Expr>) -> EvalResult<T> {
    op(arg).map_err(|arg| EvalError::InvalidArgumentTypes(vec![arg.to_string()]))
}

mod macros {
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

fn eval_eval(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    let env = env.top_level_env();
    super::eval(&expr, env)
}

fn eval_eval_local(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    super::eval(&expr, env)
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

fn eval_eq(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (a, b) = eval_2(args, env)?;
    Ok(Expr::Bool(a.lenient_eq(&b)))
}
