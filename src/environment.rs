use std::{borrow::Cow, cell::RefCell, rc::Rc};

use fnv::FnvHashMap;

use crate::{ast::Expr, eval::builtins::BUILTINS};

#[derive(Debug, Default, PartialEq)]
pub struct Environment {
    variables: RefCell<FnvHashMap<Cow<'static, str>, Expr>>,
    parent: Option<Env>,
}

pub type Env = Rc<Environment>;

impl Environment {
    pub fn top_level_env<'a>(self: &'a Env) -> &'a Env {
        std::iter::successors(Some(self), |env| env.parent.as_ref())
            .last()
            .unwrap()
    }

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
    pub fn new() -> Env {
        let env = Environment::default();
        Rc::new(env)
    }

    pub fn with_builtins() -> Env {
        let env = Self::new();
        for (builtin, _) in BUILTINS {
            env.set_special(builtin, Expr::BuiltinFunction(builtin));
        }

        env
    }

    pub fn with_parent(parent: Env) -> Env {
        let env = Environment {
            parent: Some(parent),
            ..Default::default()
        };
        Rc::new(env)
    }
}
