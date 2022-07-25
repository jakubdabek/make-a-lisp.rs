use std::borrow::Cow;

use fnv::FnvHashMap;

use crate::eval::{eval_add, eval_div, eval_mul, eval_sub, EvalFunc};

pub type Environment = FnvHashMap<Cow<'static, str>, EvalFunc>;

pub fn default_environment() -> Environment {
    let mut env = Environment::default();

    env.insert("+".into(), eval_add);
    env.insert("-".into(), eval_sub);
    env.insert("*".into(), eval_mul);
    env.insert("/".into(), eval_div);

    env
}
