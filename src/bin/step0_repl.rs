use rust2::repl::{self, Error, Result};

fn main() {
    loop {
        match rep() {
            Ok(_) => {}
            Err(Error::Eof) => break,
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
        }
    }
}

fn rep() -> Result<()> {
    let command = repl::read()?;
    let result = execute(command)?;
    let repr = print(result)?;
    println!("{repr}");
    Ok(())
}
fn execute(s: String) -> Result<String> {
    Ok(s)
}

fn print(s: String) -> Result<String> {
    Ok(s)
}
