use crate::{ast::Expr, environment::Environment};

#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    #[error("invalid function name: {0}")]
    InvalidFunctionName(String),
    #[error("invalid number of arguments")]
    InvalidArgumentsNumber,
    #[error("invalid function arguments ({0}, {1})")]
    InvalidArguments(String, String),
    #[error("unknown symbol: {0}")]
    UnknownSymbol(String),
}

pub type EvalResult<T> = std::result::Result<T, EvalError>;
pub type EvalFunc = for<'a> fn(&Expr<'a>, &Expr<'a>) -> EvalResult<Expr<'a>>;

pub fn eval<'s>(expr: Expr<'s>, env: &Environment) -> EvalResult<Expr<'s>> {
    match expr {
        Expr::List(v) => eval_list(v, env),
        Expr::Vector(v) => Ok(Expr::Vector(
            v.into_iter()
                .map(|e| eval(e, env))
                .collect::<EvalResult<_>>()?,
        )),
        Expr::Map(m) => Ok(Expr::Map(
            m.into_iter()
                .map(|e| eval(e, env))
                .collect::<EvalResult<_>>()?,
        )),
        expr => Ok(expr),
    }
}

fn eval_list<'s>(exprs: Vec<Expr<'s>>, env: &Environment) -> EvalResult<Expr<'s>> {
    if exprs.is_empty() {
        return Ok(Expr::List(exprs));
    }

    if exprs.len() != 3 {
        return Err(EvalError::InvalidArgumentsNumber);
    }

    let mut exprs = exprs.into_iter();
    let [name, a, b] = [(); 3].map(|_| exprs.next().unwrap());
    let name = eval(name, env)?;

    let name = match name {
        Expr::Symbol(s) => s,
        _ => return Err(EvalError::InvalidFunctionName(name.to_string())),
    };

    let f = match env.get(&*name) {
        Some(f) => f,
        None => return Err(EvalError::UnknownSymbol(name.into_owned())),
    };

    let a = eval(a, env)?;
    let b = eval(b, env)?;

    f(&a, &b)
}

fn number_args<'s>(a: &Expr<'s>, b: &Expr<'s>) -> EvalResult<(i64, i64)> {
    match (a, b) {
        (Expr::Int(a), Expr::Int(b)) => Ok((*a, *b)),
        _ => Err(EvalError::InvalidArguments(a.to_string(), b.to_string())),
    }
}

macro_rules! def_arithmetic {
    ($($fn:ident $op:tt),+ $(,)?) => {
        $(
        pub fn $fn<'s>(a: &Expr<'s>, b: &Expr<'s>) -> EvalResult<Expr<'s>> {
            let (a, b) = number_args(a, b)?;
            Ok(Expr::Int(a $op b))
        }
        )+
    };
}

def_arithmetic! {
    eval_add +,
    eval_sub -,
    eval_mul *,
    eval_div /,
}
