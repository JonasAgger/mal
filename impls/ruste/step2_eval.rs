use anyhow::Result;
use environment::Environment;
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
    while let Some(input) = console::Console::read_user_input() {
        let lexer = Lexer::tokenize(&input);
        let mut parser = Parser::new(lexer);
        let mut env = Environment::new();
        let tokens = parser.parse();

        match tokens {
            Ok(tokens) => {
                dbg!(&tokens);

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

fn eval(ast: &MalType, env: &mut Environment) -> Result<MalType> {
    match ast {
        MalType::List(inner) => {
            if inner.len() == 0 {
                return Ok(ast.clone());
            }
            match eval_ast(ast, env)? {
                MalType::List(inner) => {
                    let func = inner[0].clone();
                    func.eval(&inner[1..], env)
                }
                _ => anyhow::bail!("Expected a list"),
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_ast(ast: &MalType, env: &mut Environment) -> Result<MalType> {
    match ast {
        MalType::List(inner) => {
            let mut list = vec![];
            for item in inner {
                list.push(eval(item, env)?);
            }
            Ok(MalType::List(list))
        }
        MalType::HashMap(map) => {
            let mut list = vec![];
            for item in map.windows(2) {
                list.push(item[0].clone());
                list.push(eval(&item[1], env)?);
            }
            Ok(MalType::HashMap(list))
        }
        MalType::Vector(inner) => {
            let mut list = vec![];
            for item in inner {
                list.push(eval(item, env)?);
            }
            Ok(MalType::Vector(list))
        }
        MalType::Symbol(sym) => env
            .get(sym)
            .ok_or(anyhow::anyhow!("symbol not found: {}", sym)),
        _ => Ok(ast.clone()),
    }
}
