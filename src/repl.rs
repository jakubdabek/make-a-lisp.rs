use std::io::{self, Write};

use crate::{
    ast::Expr,
    environment::{Env, Environment},
    eval::{self, EvalError},
    parser::{self, ParseError},
};

use self::repl_funcs::{NoPrint, ReplFuncs, WithStaticInput};

pub mod repl_funcs;

pub fn main(funcs: impl ReplFuncs) {
    let mut args = std::env::args().skip(1);
    match args.next() {
        Some(program) => repl(NoPrint(WithStaticInput::new(
            std::iter::once(format!("(load-file {:?})", program)),
            funcs,
        ))),
        None => repl(funcs),
    }
}

pub fn repl(funcs: impl ReplFuncs) {
    let env = define_builtins(&funcs);

    if funcs.is_interactive() {
        funcs
            .execute(r##"(println (str "Mal [" *host-language* "]"))"##, &env)
            .unwrap();
    }

    loop {
        match rep(&funcs, &env) {
            Ok(_) => {}
            Err(Error::Eof) => break,
            Err(Error::Parse(ParseError::Empty)) => {
                if funcs.is_interactive() {
                    println!();
                }
            }
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

pub fn define_builtins(funcs: &impl ReplFuncs) -> Env {
    let env = Environment::with_builtins();
    funcs
        .execute("(def! not (fn* [arg] (if arg false true)))", &env)
        .unwrap();

    funcs
        .execute(
            r##"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"##,
            &env,
        )
        .unwrap();

    funcs
        .execute(
            r##"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"##,
            &env,
        )
        .unwrap();

    env.set_special(
        "*ARGV*",
        Expr::List(std::env::args().skip(2).map(Expr::String).collect()),
    );

    env.set_special("*host-language*", Expr::String("rust2".into()));

    env
}

pub fn rep(funcs: &impl ReplFuncs, env: &Env) -> Result<()> {
    let command = funcs.read()?;
    let result = funcs.execute(&command, env)?;
    let repr = funcs.print(result)?;
    print!("{repr}");
    std::io::stdout().flush().unwrap();
    Ok(())
}

pub fn prompt(pr: Option<&str>) {
    print!("{}", pr.unwrap_or("user> "));
    std::io::stdout().flush().unwrap();
}

pub fn read(pr: Option<&str>) -> Result<String> {
    prompt(pr);
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
    Ok(format!("{:#}\n", expr))
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("end of input")]
    Eof,
    #[error("empty string")]
    Empty,
    #[error("reading input failed: {0}")]
    IO(#[from] io::Error),
    #[error("parsing failed: {0}")]
    Parse(#[from] ParseError),
    #[error("evaluation failed: {0}")]
    Eval(#[from] EvalError),
}
