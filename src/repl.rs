use std::io::{self, Write};

use crate::{
    ast::Expr,
    environment::{Env, Environment},
    eval::{self, EvalError},
    parser::{self, ParseError},
};

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

pub type Result<T> = std::result::Result<T, Error>;

pub fn main<Execute>(exec: Execute)
where
    Execute: Copy + Fn(&str, &Env) -> Result<Expr>,
{
    let env = Environment::with_builtins();
    loop {
        match rep(exec, &env) {
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

pub fn rep<Execute>(exec: Execute, env: &Env) -> Result<()>
where
    Execute: Fn(&str, &Env) -> Result<Expr>,
{
    let command = read()?;
    let result = exec(&command, env)?;
    let repr = print(result)?;
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
