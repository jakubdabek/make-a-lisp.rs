use crate::{ast::Expr, environment::Env};

use super::Result;

pub trait ReplFuncs {
    type Value;
    fn read(&self) -> Result<String> {
        super::read()
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
