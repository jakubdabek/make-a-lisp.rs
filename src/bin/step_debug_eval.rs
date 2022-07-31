use rust2::{
    ast::Expr,
    environment::Env,
    repl::{self, repl_funcs::ReplFuncs, Result},
};

#[derive(Debug, Clone, Copy)]
pub struct DebugEval;

impl ReplFuncs for DebugEval {
    type Value = Expr;

    fn execute(&self, s: &str, env: &Env) -> Result<Expr> {
        repl::execute_eval(s, env)
    }

    fn print(&self, expr: Expr) -> Result<String> {
        Ok(format!("{:?}", expr))
    }
}

fn main() {
    repl::main(DebugEval)
}
