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
    mal_define_fn(&mut env)?;
    setup();

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

#[cfg(debug_assertions)]
fn setup() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();
}

#[cfg(not(debug_assertions))]
fn setup() {}

fn mal_define_fn(env: &mut Environment) -> Result<()> {
    let input = String::from("(def! not (fn* (a) (if a false true)))");

    let lexer = Lexer::tokenize(&input);
    let mut parser = Parser::new(lexer);
    let tokens = parser.parse()?;

    eval(tokens.first().unwrap(), env)?;
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
