use std::{borrow::Cow, cell::RefCell, rc::Rc};

use fnv::FnvHashMap;

use crate::{
    ast::Expr,
    eval::{eval_add, eval_div, eval_mul, eval_sub, EvalFunc},
};

#[derive(Debug, Default)]
pub struct Environment {
    variables: RefCell<FnvHashMap<Cow<'static, str>, Expr>>,
    parent: Option<Env>,
}

pub type Env = Rc<Environment>;

impl Environment {
    pub fn get(&self, name: &str) -> Option<Expr> {
        self.variables
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref()?.get(name))
    }

    pub fn set(&self, name: &str, expr: Expr) {
        self.set_cow(Cow::Owned(name.to_owned()), expr)
    }

    pub fn set_special(&self, name: &'static str, expr: Expr) {
        self.set_cow(Cow::Borrowed(name), expr)
    }

    fn set_cow(&self, name: Cow<'static, str>, expr: Expr) {
        self.variables.borrow_mut().insert(name, expr);
    }
}

impl Environment {
    pub fn with_builtins() -> Env {
        let env = Environment::default();

        env.set_special("+", Expr::Function(EvalFunc(eval_add)));
        env.set_special("-", Expr::Function(EvalFunc(eval_sub)));
        env.set_special("*", Expr::Function(EvalFunc(eval_mul)));
        env.set_special("/", Expr::Function(EvalFunc(eval_div)));

        Rc::new(env)
    }

    pub fn with_parent(parent: Env) -> Env {
        let env = Environment {
            parent: Some(parent),
            ..Default::default()
        };
        Rc::new(env)
    }
}
