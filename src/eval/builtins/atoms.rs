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
    let expr = as_type!(&expr => Expr::Atom)?.borrow().clone();
    Ok(expr)
}

pub(super) fn eval_reset(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (atom, expr) = eval_2(args, env)?;
    let atom = as_type!(&atom => Expr::Atom)?;

    *atom.borrow_mut() = expr.clone();
    Ok(expr)
}

pub(super) fn eval_swap(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let mut args = eval_args(args, env)?;
    let atom_ref = match &mut args[..] {
        [atom, func, ..] => {
            std::mem::swap(atom, func);
            let (atom, _func) = (func, atom);
            let atom_ref = as_type!(atom => Expr::Atom)?.clone();
            *atom = atom_ref.borrow().clone();
            atom_ref
        }
        _ => return Err(EvalError::InvalidArgumentCount),
    };

    let value = eval::eval(&Expr::List(args), env)?;
    *atom_ref.borrow_mut() = value.clone();
    Ok(value)
}
