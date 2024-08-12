use crate::{environment::Environment, types::MalType};
use anyhow::Result;
use log::{debug, trace};

pub fn eval(ast: &MalType, env: &mut Environment) -> Result<MalType> {
    debug!("Eval: ast: {:?}", ast);
    trace!("Eval: ast: {:?}, env: {:?}", ast, env);
    let ret = match ast {
        MalType::List(inner) => {
            if inner.len() == 0 {
                trace!("Eval: empty list");
                return Ok(ast.clone());
            }

            let bind = {
                if let MalType::Symbol(sym) = &inner[0] {
                    env.get(sym)
                } else {
                    None
                }
            };

            trace!("Eval: bind: {:?}", bind);
            if let Some(MalType::Bind(expr)) = bind {
                trace!("Eval: eval bind: {:?}", expr);
                expr.eval(&inner[1..], env)
            } else {
                trace!("Eval: eval list");
                match eval_ast(ast, env)? {
                    MalType::List(inner) => {
                        trace!("Eval->eval_ast: eval list: {:?}", inner);
                        let func = inner[0].clone();
                        func.eval(&inner[1..], env)
                    }
                    _ => anyhow::bail!("Expected a list"),
                }
            }
        }
        _ => eval_ast(ast, env),
    };
    trace!("Eval: ret: {:?}", ret);
    ret
}

fn eval_ast(ast: &MalType, env: &mut Environment) -> Result<MalType> {
    debug!("EvalAst: ast: {:?}", ast);
    trace!("EvalAst: ast: {:?}, env: {:?}", ast, env);
    let ret = match ast {
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
        MalType::Symbol(sym) => env.get(sym).ok_or(anyhow::anyhow!("{} not found", sym)),
        _ => Ok(ast.clone()),
    };
    trace!("EvalAst: ret: {:?}", ret);
    ret
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::reader::{Lexer, Parser};

    use super::*;

    #[test]
    fn apply_list() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        let ast = MalType::List(vec![MalType::Symbol(String::from("list"))]);
        let mut env = Environment::new();

        let r = eval(&ast, &mut env).unwrap();

        assert_eq!(r, MalType::List(vec![]))
    }

    #[test]
    fn test_closures() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        // (fn* (b) (+ a b))
        // (fn* (a) (fn* (b) (+ a b)))
        // (  5)
        // (  7)
        let lexer = Lexer::tokenize("( ( (fn* (a) (fn* (b) (+ a b))) 5) 7)");
        let mut parser = Parser::new(lexer);

        let ast = parser.parse().unwrap();
        let mut env = Environment::new();

        let r = eval(&ast[0], &mut env).unwrap();

        assert_eq!(r, MalType::Number(12))
    }

    #[test]
    fn pr_str() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .init();
        let ast = MalType::List(vec![MalType::Symbol(String::from("pr-str"))]);
        let mut env = Environment::new();

        let r = eval(&ast, &mut env).unwrap();

        assert_eq!(r, MalType::String(String::new()))
    }
}
