use super::prelude::*;

pub(super) fn eval_map(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (f, list) = eval_2(args, env)?;
    let list = into_type(list, Expr::into_list_like)?;

    let results = list
        .into_iter()
        .map(|elem| super::eval(&Expr::List(vec![f.clone(), elem]), env))
        .collect::<EvalResult<_>>()?;
    Ok(Expr::List(results))
}

pub(super) fn eval_apply(args: &[Expr], env: &Env) -> EvalResult<Expr> {
    let (f, args) = args.split_first().ok_or(EvalError::InvalidArgumentCount)?;
    let (list, args) = args.split_last().ok_or(EvalError::InvalidArgumentCount)?;
    let list = super::eval(list, env)?;
    let list = into_type(list, Expr::into_list_like)?;
    let mut f_args = vec![f.clone()];
    f_args.extend(args.iter().cloned());
    f_args.extend(
        list.into_iter()
            .map(|e| Expr::List(vec![Expr::BuiltinFunction("quote"), e])),
    );

    super::eval(&Expr::List(f_args), env)
}
