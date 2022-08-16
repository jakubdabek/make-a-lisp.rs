use std::cell::RefCell;

use crate::{ast::Expr, environment::Env};

use super::{Error, Result};

pub trait ReplFuncs {
    type Value;
    fn is_interactive(&self) -> bool {
        true
    }
    fn read(&self) -> Result<String> {
        super::read(None)
    }
    fn execute(&self, s: &str, env: &Env) -> Result<Self::Value>;
    fn print(&self, expr: Self::Value) -> Result<String>;
}

#[derive(Debug, Clone, Copy)]
pub struct WithEval;

impl ReplFuncs for WithEval {
    type Value = Expr;

    fn execute(&self, s: &str, env: &Env) -> Result<Expr> {
        super::execute_eval(s, env)
    }

    fn print(&self, expr: Expr) -> Result<String> {
        super::print(expr)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WithoutEval;

impl ReplFuncs for WithoutEval {
    type Value = Expr;

    fn execute(&self, s: &str, env: &Env) -> Result<Expr> {
        super::execute_no_eval(s, env)
    }

    fn print(&self, expr: Expr) -> Result<String> {
        super::print(expr)
    }
}

#[derive(Debug, Clone)]
pub struct WithStaticInput<I, Funcs>(RefCell<I>, Funcs);

impl<I, Funcs> WithStaticInput<I, Funcs> {
    pub fn new(input: I, funcs: Funcs) -> Self {
        Self(RefCell::new(input), funcs)
    }
}

impl<I, Funcs: ReplFuncs> ReplFuncs for WithStaticInput<I, Funcs>
where
    I: Iterator<Item = String>,
{
    type Value = Funcs::Value;

    fn is_interactive(&self) -> bool {
        false
    }

    fn read(&self) -> Result<String> {
        self.0.borrow_mut().next().ok_or(Error::Eof)
    }

    fn execute(&self, s: &str, env: &Env) -> Result<Self::Value> {
        self.1.execute(s, env)
    }

    fn print(&self, expr: Self::Value) -> Result<String> {
        self.1.print(expr)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoPrint<Funcs>(pub Funcs);

impl<Funcs: ReplFuncs> ReplFuncs for NoPrint<Funcs> {
    type Value = Funcs::Value;

    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }

    fn read(&self) -> Result<String> {
        self.0.read()
    }

    fn execute(&self, s: &str, env: &Env) -> Result<Self::Value> {
        self.0.execute(s, env)
    }

    fn print(&self, _expr: Self::Value) -> Result<String> {
        Ok(String::new())
    }
}
