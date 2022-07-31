use std::io::{self, Write};

use crate::{
    ast::Expr,
    environment::{Env, Environment},
    eval::{self, EvalError},
    parser::{self, ParseError},
};

use self::repl_funcs::ReplFuncs;

pub mod repl_funcs;

pub fn main(funcs: impl ReplFuncs) {
    let env = Environment::with_builtins();
    loop {
        match rep(&funcs, &env) {
            Ok(_) => {}
            Err(Error::Eof) => break,
            Err(Error::Parse(e)) => {
                println!("Error: {e}");
            }
            Err(Error::Eval(e)) => {
                println!("Error: {e}");
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }
}

pub fn rep(funcs: &impl ReplFuncs, env: &Env) -> Result<()> {
    let command = funcs.read()?;
    let result = funcs.execute(&command, env)?;
    let repr = funcs.print(result)?;
    println!("{repr}");
    Ok(())
}

pub fn prompt() {
    print!("user> ");
    std::io::stdout().flush().unwrap();
}

pub fn read() -> Result<String> {
    prompt();
    let mut s = String::new();
    if std::io::stdin().read_line(&mut s)? == 0 {
        Err(Error::Eof)
    } else {
        let new_len = s.trim_end_matches(&['\n', '\r']).len();
        s.truncate(new_len);
        Ok(s)
    }
}

pub fn execute_eval(s: &str, env: &Env) -> Result<Expr> {
    let expr = parser::parse(s)?;
    Ok(eval::eval(&expr, env)?)
}

pub fn execute_no_eval(s: &str, _env: &Env) -> Result<Expr> {
    let expr = parser::parse(s)?;
    Ok(expr)
}

pub fn print(expr: Expr) -> Result<String> {
    Ok(format!("{:#}", expr))
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("end of input")]
    Eof,
    #[error("reading input failed: {0}")]
    IO(#[from] io::Error),
    #[error("parsing failed: {0}")]
    Parse(#[from] ParseError),
    #[error("evaluation failed: {0}")]
    Eval(#[from] EvalError),
}
