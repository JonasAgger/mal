use anyhow::Result;
use environment::Environment;
use eval::eval;
use reader::{Lexer, Parser};
use types::MalType;
mod console;
mod core;
mod environment;
mod eval;
mod expr;
mod reader;
mod types;

fn main() -> Result<()> {
    let mut env = Environment::new();
    while let Some(input) = console::Console::read_user_input() {
        let lexer = Lexer::tokenize(&input);
        let mut parser = Parser::new(lexer);
        let tokens = parser.parse();

        match tokens {
            Ok(tokens) => {
                rep(tokens, &mut env)?;
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }

        // println!("{}", input);
    }

    Ok(())
}

fn rep(tokens: Vec<MalType>, env: &mut Environment) -> Result<()> {
    let token = tokens.first().unwrap();

    match eval(token, env) {
        Ok(exp) => {
            println!("{}", exp);
        }
        Err(err) => {
            println!("Error: {:?}", err);
        }
    }

    Ok(())
}
