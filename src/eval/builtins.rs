use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ast::Expr, environment::Env};

use super::{
    eval,
    utils::*,
    EvalResult,
    Thunk::{self, Evaluated},
};

mod atoms;
mod control_flow;
mod functional;
mod lists;
mod maps;
mod meta;
mod primitives;
mod quoting;
mod strings;

mod prelude {
    pub(super) use super::super::utils::{macros::*, *};
    pub(super) use crate::{
        ast::Expr,
        environment::{Env, Environment},
        eval::{self, EvalError, EvalResult},
    };
}

use self::{
    atoms::*, control_flow::*, functional::*, lists::*, maps::*, meta::*, primitives::*,
    quoting::*, strings::*,
};
pub use maps::list_to_hash_map;

// const ARITHMETIC_BUILTINS: &[&str] = &["+", "-", "*", "/"];
// const COMPARISON_BUILTINS: &[&str] = &["<", ">", ">=", "<="];

pub type BuiltinThunkFn = fn(&[Expr], &Env) -> EvalResult<Thunk>;
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
    ("try*", eval_try),
    ("throw", eval_throw),
    ("time-ms", eval_time_ms),
    // meta
    ("meta", eval_meta),
    ("with-meta", eval_with_meta),
    // functional
    ("map", eval_map),
    ("apply", eval_apply),
    // lists
    ("list", eval_list),
    ("list?", eval_is_list),
    ("sequential?", eval_is_sequential),
    ("empty?", eval_is_empty),
    ("count", eval_count),
    ("cons", eval_cons),
    ("first", eval_first),
    ("rest", eval_rest),
    ("nth", eval_nth),
    ("concat", eval_concat),
    ("vec", eval_vec),
    ("vector", eval_vector),
    ("vector?", eval_is_vector),
    ("conj", eval_conj),
    ("seq", eval_seq),
    // maps
    ("map?", eval_is_map),
    ("hash-map", eval_hash_map),
    ("keys", eval_keys),
    ("vals", eval_vals),
    ("get", eval_get),
    ("assoc", eval_assoc),
    ("dissoc", eval_dissoc),
    ("contains?", eval_contains),
    // strings
    ("pr-str", eval_pr_str),
    ("str", eval_str),
    ("prn", eval_prn),
    ("println", eval_println),
    ("slurp", eval_slurp),
    ("read-string", eval_read_string),
    ("readline", eval_readline),
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
    // primitives
    ("false?", eval_is_false),
    ("true?", eval_is_true),
    ("nil?", eval_is_nil),
    ("string?", eval_is_string),
    ("fn?", eval_is_fn),
    ("macro?", eval_is_macro),
    ("number?", eval_is_number),
    ("symbol?", eval_is_symbol),
    ("symbol", eval_symbol),
    ("keyword?", eval_is_keyword),
    ("keyword", eval_keyword),
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

pub const THUNK_BUILTINS: &[(&str, BuiltinThunkFn)] = &[
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

fn eval_time_ms(args: &[Expr], _env: &Env) -> EvalResult<Expr> {
    let [] = args_n(args)?;
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    Ok(Expr::Int(since_the_epoch.as_millis() as i64))
}
