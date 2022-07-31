use rust2::{
    environment::Env,
    repl::{self, repl_funcs::ReplFuncs, Result},
};

#[derive(Debug, Clone, Copy)]
pub struct Step0;

impl ReplFuncs for Step0 {
    type Value = String;

    fn execute(&self, s: &str, _env: &Env) -> Result<String> {
        Ok(s.to_owned())
    }

    fn print(&self, expr: String) -> Result<String> {
        Ok(expr)
    }
}

fn main() {
    repl::main(Step0)
}
