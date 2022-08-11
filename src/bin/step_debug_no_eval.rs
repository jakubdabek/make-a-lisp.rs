use rust2::{
    ast::Expr,
    environment::Env,
    repl::{self, repl_funcs::ReplFuncs, Result},
};

#[derive(Debug, Clone, Copy)]
pub struct DebugNoEval;

impl ReplFuncs for DebugNoEval {
    type Value = Expr;

    fn execute(&self, s: &str, env: &Env) -> Result<Expr> {
        repl::execute_no_eval(s, env)
    }

    fn print(&self, expr: Expr) -> Result<String> {
        Ok(format!("{:?}\n", expr))
    }
}

fn main() {
    repl::main(DebugNoEval)
}
