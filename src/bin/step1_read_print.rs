#![deny(rust_2018_idioms)]

use std::io::{self, Write};

use rust2::{
    ast::Expr,
    parser::{self, ParseError},
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("end of input")]
    Eof,
    #[error("reading input failed: {0}")]
    IO(#[from] io::Error),
    #[error("parsing failed: {0}")]
    Parse(#[from] ParseError),
}

type Result<T> = std::result::Result<T, Error>;

fn main() {
    loop {
        match rep() {
            Ok(_) => {}
            Err(Error::Eof) => break,
            Err(Error::Parse(e)) => {
                println!("Error: {e}");
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }
}

fn rep() -> Result<()> {
    let command = read()?;
    let result = execute(&command)?;
    let repr = print(result)?;
    println!("{repr}");
    Ok(())
}

fn prompt() {
    print!("user> ");
    std::io::stdout().flush().unwrap();
}

fn read() -> Result<String> {
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

fn execute(s: &str) -> Result<Expr<'_>> {
    Ok(parser::parse(s)?)
}

fn print(expr: Expr<'_>) -> Result<String> {
    Ok(format!("{:?}", expr))
}
