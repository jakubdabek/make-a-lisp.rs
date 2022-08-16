use std::rc::Rc;

use itertools::Itertools;

use crate::ast::{Map, MapKey};

use super::prelude::*;

pub(super) fn eval_is_map(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    is_type!(args, env, Expr::Map(_))
}

pub(super) fn eval_hash_map(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;
    list_to_hash_map(&args).map(Rc::new).map(Expr::Map)
}

pub fn list_to_hash_map(list: &[Expr]) -> EvalResult<Map> {
    if list.len() % 2 != 0 {
        return Err(EvalError::InvalidArgumentCount);
    }

    let map = list
        .iter()
        .tuples()
        .map(|(k, v)| Ok((as_type(k, Expr::to_map_key)?, v.clone())))
        .collect::<EvalResult<_>>()?;

    Ok(map)
}

pub(super) fn eval_keys(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    let list = as_type!(&expr => Expr::Map)?;
    Ok(Expr::List(list.keys().map(MapKey::to_expr).collect()))
}

pub(super) fn eval_vals(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    let map = as_type!(&expr => Expr::Map)?;
    Ok(Expr::List(map.values().cloned().collect()))
}

pub(super) fn eval_get(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (map, key) = eval_2(args, env)?;
    let map = as_type!(&map => Expr::Map)?;
    let key = match key.to_map_key() {
        Some(k) => k,
        None => return Ok(Expr::Nil),
    };

    Ok(map.get(&key).cloned().unwrap_or(Expr::Nil))
}

pub(super) fn eval_contains(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (map, key) = eval_2(args, env)?;
    let map = as_type!(&map => Expr::Map)?;
    let key = match key.to_map_key() {
        Some(k) => k,
        None => return Ok(Expr::Bool(false)),
    };

    Ok(Expr::Bool(map.contains_key(&key)))
}

pub(super) fn eval_assoc(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;
    let (map, args) = args.split_first().ok_or(EvalError::InvalidArgumentCount)?;
    let map = as_type!(map => Expr::Map)?;
    let args_map = list_to_hash_map(args)?;
    let mut new_map = Map::clone(map);
    new_map.extend(args_map);

    Ok(Expr::Map(Rc::new(new_map)))
}

pub(super) fn eval_dissoc(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let args = eval_args(args, env)?;
    let (map, args) = args.split_first().ok_or(EvalError::InvalidArgumentCount)?;
    let map = as_type!(map => Expr::Map)?;
    let mut new_map = Map::clone(map);
    for arg in args {
        match arg.to_map_key() {
            Some(k) => new_map.remove(&k),
            None => continue,
        };
    }

    Ok(Expr::Map(Rc::new(new_map)))
}
