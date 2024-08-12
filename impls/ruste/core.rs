use anyhow::Result;
use log::{debug, trace};
use std::{collections::HashMap, rc::Rc};

use crate::{
    environment::Environment,
    types::{MalExpr, MalFn, MalLibFn, MalType},
};

pub fn add_functions(hm: &mut HashMap<String, MalType>) {
    // Keywords
    make_bind(hm, "def!", def);
    make_bind(hm, "let*", _let);
    make_bind(hm, "do", _do);
    make_bind(hm, "if", _if);
    make_bind(hm, "fn*", _fn);

    // core functions
    make_fn(hm, "pr-str", pr_str);
    make_fn(hm, "str", str);
    make_fn(hm, "prn", prn);
    make_fn(hm, "println", println);

    make_fn(hm, "list", list);
    make_fn(hm, "list?", is_list);
    make_fn(hm, "empty?", any);
    make_fn(hm, "count", count);

    // Arithmic
    make_bin_op(hm, "+", |val1, val2| val1 + val2);
    make_bin_op(hm, "-", |val1, val2| val1 - val2);
    make_bin_op(hm, "*", |val1, val2| val1 * val2);
    make_bin_op(hm, "/", |val1, val2| val1 / val2);

    // Cmp
    make_bin_op(hm, "=", |val1, val2| MalType::Bool(val1 == val2));
    make_bin_op(hm, "<", |val1, val2| MalType::Bool(val1 < val2));
    make_bin_op(hm, "<=", |val1, val2| MalType::Bool(val1 <= val2));
    make_bin_op(hm, ">", |val1, val2| MalType::Bool(val1 > val2));
    make_bin_op(hm, ">=", |val1, val2| MalType::Bool(val1 >= val2));
}

fn count(args: &[MalType], _: Environment) -> Result<MalType> {
    match &args[0] {
        MalType::List(inner) => Ok(MalType::Number(inner.len() as i64)),
        MalType::Nil => Ok(MalType::Number(0)),
        _ => anyhow::bail!("count? received unexpected value {:?}", &args[0]),
    }
}

fn any(args: &[MalType], _: Environment) -> Result<MalType> {
    match &args[0] {
        MalType::List(inner) => Ok(MalType::Bool(inner.len() == 0)),
        _ => anyhow::bail!("empty? received unexpected value {:?}", &args[0]),
    }
}

fn is_list(args: &[MalType], _: Environment) -> Result<MalType> {
    match &args[0] {
        MalType::List(_) => Ok(MalType::Bool(true)),
        _ => Ok(MalType::Bool(false)),
    }
}
fn list(args: &[MalType], _: Environment) -> Result<MalType> {
    Ok(MalType::List(args.to_vec()))
}

// Pretty
fn prn(args: &[MalType], _: Environment) -> Result<MalType> {
    if let Some(arg) = args.get(0) {
        println!("{:b}", arg);
    } else {
        println!()
    }
    Ok(MalType::Nil)
}

fn println(args: &[MalType], _: Environment) -> Result<MalType> {
    use std::fmt::Write;
    let mut buffer = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            write!(&mut buffer, " {}", arg)?;
        } else {
            write!(&mut buffer, "{}", arg)?;
        }
    }
    println!("{}", buffer);
    Ok(MalType::Nil)
}

// Pretty
fn pr_str(args: &[MalType], _: Environment) -> Result<MalType> {
    use std::fmt::Write;
    let mut buffer = String::new();
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            write!(&mut buffer, " {:b}", arg)?;
        } else {
            write!(&mut buffer, "{:b}", arg)?;
        }
    }
    Ok(MalType::String(buffer))
}

fn str(args: &[MalType], _: Environment) -> Result<MalType> {
    use std::fmt::Write;
    let mut buffer = String::new();
    for (i, arg) in args.iter().enumerate() {
        write!(&mut buffer, "{}", arg)?;
    }
    Ok(MalType::String(buffer))
}

fn _do(args: &[MalType], mut env: Environment) -> Result<MalType> {
    // eval all items in list, return last
    args.iter()
        .map(|item| crate::eval::eval(item, &mut env))
        .last()
        .ok_or(anyhow::anyhow!("Received error while trying to eval?"))?
}

fn _if(args: &[MalType], mut env: Environment) -> Result<MalType> {
    let booleanish = crate::eval::eval(&args[0], &mut env)?;

    // if trueish, eval 2nd parameter, otherwise eval third
    match booleanish {
        MalType::Nil => {
            if args.len() < 3 {
                return Ok(MalType::Nil);
            }
            crate::eval::eval(&args[2], &mut env)
        }
        MalType::Bool(x) if !x => {
            if args.len() < 3 {
                return Ok(MalType::Nil);
            }
            crate::eval::eval(&args[2], &mut env)
        }
        _ => crate::eval::eval(&args[1], &mut env),
    }
}

fn _fn(args: &[MalType], env: Environment) -> Result<MalType> {
    match &args[0] {
        MalType::List(_) => (),
        _ => anyhow::bail!("Received non list as parameter to fn*"),
    };

    let mal_fn = MalFn {
        expr: Box::new(args[1].clone()),
        captured_args: Box::new(args[0].clone()),
        captured_env: env,
    };
    Ok(MalType::Fn(mal_fn))
}

fn def(args: &[MalType], mut env: Environment) -> Result<MalType> {
    let eval = crate::eval::eval(&args[1], &mut env)?;
    env.set(args[0].clone(), eval.clone());
    Ok(eval)
}

fn _let(args: &[MalType], mut env: Environment) -> Result<MalType> {
    env.enter();
    match &args[0] {
        MalType::List(inner) => {
            for i in (0..inner.len()).step_by(2) {
                let eval = crate::eval::eval(&inner[i + 1], &mut env)?;
                env.set(inner[i].clone(), eval);
            }
        }
        MalType::Vector(inner) => {
            for i in (0..inner.len()).step_by(2) {
                let eval = crate::eval::eval(&inner[i + 1], &mut env)?;
                env.set(inner[i].clone(), eval);
            }
        }
        other => panic!(
            "Let binding received not list as first parameter: {:?}",
            other
        ),
    }

    let ret = crate::eval::eval(args.last().unwrap(), &mut env);
    env.exit();
    ret
}

fn make_bin_op(hm: &mut HashMap<String, MalType>, s: &str, f: fn(&MalType, &MalType) -> MalType) {
    let inner_fn = Rc::new(move |x: &[MalType], _| match x {
        [arg1, arg2, ..] => Ok(f(arg1, arg2)),
        _ => unreachable!(),
    });

    let op = MalType::BinOp(MalExpr {
        symbol: s.to_string(),
        arguments: 2,
        inner: inner_fn,
    });

    hm.insert(s.to_string(), op);
}

fn make_bind(
    hm: &mut HashMap<String, MalType>,
    s: &'static str,
    f: fn(&[MalType], Environment) -> Result<MalType>,
) {
    let inner_fn = Rc::new(move |x: &[MalType], env| {
        debug!("{}", s);
        trace!("{}. args: {:?}", s, x);
        let ret = f(x, env);
        trace!("{}. ret: {:?}", s, ret);
        ret
    });

    let op = MalType::Bind(MalExpr {
        symbol: s.to_string(),
        arguments: 0,
        inner: inner_fn,
    });

    hm.insert(s.to_string(), op);
}

fn make_fn(
    hm: &mut HashMap<String, MalType>,
    s: &'static str,
    f: fn(&[MalType], Environment) -> Result<MalType>,
) {
    let inner_fn = Rc::new(move |x: &[MalType], env| {
        debug!("{}", s);
        trace!("{}. args: {:?}", s, x);
        let ret = f(x, env);
        trace!("{}. ret: {:?}", s, ret);
        ret
    });

    let op = MalExpr {
        symbol: s.to_string(),
        arguments: 0,
        inner: inner_fn,
    };

    let _fn = MalType::LibFn(MalLibFn {
        expr: Box::new(op),
        captured_env: Box::new(MalType::List(vec![])),
    });

    hm.insert(s.to_string(), _fn);
}
