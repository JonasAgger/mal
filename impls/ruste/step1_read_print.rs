use anyhow::Result;
use reader::{Lexer, Parser};
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

        let types = parser.parse();

        match types {
            Ok(types) => {
                dbg!(&types);
                for mal_type in types {
                    println!("{}", mal_type);
                }
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }

        // println!("{}", input);
    }

    Ok(())
}
