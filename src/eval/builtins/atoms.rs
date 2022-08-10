use super::prelude::*;

pub(super) fn eval_atom(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    Ok(Expr::atom(expr))
}

pub(super) fn eval_is_atom(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    Ok(Expr::Bool(matches!(expr, Expr::Atom(_))))
}

pub(super) fn eval_deref(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let expr = eval_1(args, env)?;
    match expr {
        Expr::Atom(a) => Ok(a.borrow().clone()),
        _ => Err(EvalError::InvalidArgumentTypes(vec![expr.to_string()])),
    }
}

pub(super) fn eval_reset(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (atom, expr) = eval_2(args, env)?;
    let atom = match atom {
        Expr::Atom(a) => a,
        _ => return Err(EvalError::InvalidArgumentTypes(vec![atom.to_string()])),
    };

    *atom.borrow_mut() = expr.clone();
    Ok(expr)
}

pub(super) fn eval_swap(args: &[Expr], env: &Env) -> EvalResult<Expr> {
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

    let value = eval::eval(&Expr::List(args), env)?;
    *atom_ref.borrow_mut() = value.clone();
    Ok(value)
}
