mod console;
mod core;
mod environment;
mod eval;
mod expr;
mod reader;
mod types;

fn main() {
    while let Some(input) = console::Console::read_user_input() {
        println!("{}", input);
    }
}
