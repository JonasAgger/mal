use crate::environment::Environment;
use anyhow::Result;
use log::debug;
use std::{
    fmt::{Binary, Debug, Display},
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum MalType {
    List(Vec<MalType>),
    HashMap(Vec<MalType>),
    Vector(Vec<MalType>),
    String(String),
    Symbol(String),
    Number(i64),
    Bool(bool),
    Nil,
    Bind(MalExpr),
    BinOp(MalExpr),
    Fn(MalFn),
    LibFn(MalLibFn),
}

impl MalType {
    pub fn eval(self, val: &[MalType], env: &Environment) -> Result<MalType> {
        match self {
            MalType::Fn(expr) => expr.eval(val, env),
            MalType::LibFn(expr) => expr.eval(val, env),
            MalType::BinOp(expr) => expr.eval(val, env),
            MalType::Symbol(symbol) => env
                .get(&symbol)
                .ok_or(anyhow::anyhow!("MalType::eval: Expected to find symbol")),
            other => Ok(other),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MalFn {
    pub expr: Box<MalType>,
    pub captured_args: Box<MalType>,
    pub captured_env: Environment,
}

impl MalFn {
    pub fn eval(&self, val: &[MalType], _: &Environment) -> Result<MalType> {
        debug!("MalFn::eval: self: {:?} -- values {:?}", self, val);
        let mut env = Environment::from(
            self.captured_env.clone(),
            self.captured_args.as_ref().clone(),
            val,
        );
        crate::eval::eval(&self.expr, &mut env)
    }
}

#[derive(Clone, Debug)]
pub struct MalLibFn {
    pub expr: Box<MalExpr>,
    pub captured_env: Box<MalType>,
}

impl MalLibFn {
    pub fn eval(&self, val: &[MalType], env: &Environment) -> Result<MalType> {
        debug!("MalLibFn::eval: self: {:?} -- values {:?}", self, val);
        let env = Environment::from(env.clone(), self.captured_env.as_ref().clone(), val);
        self.expr.eval(val, &env)
    }
}

#[derive(Clone)]
pub struct MalExpr {
    pub symbol: String,
    pub arguments: usize,
    pub inner: Rc<dyn Fn(&[MalType], Environment) -> Result<MalType> + 'static>,
}

impl MalExpr {
    pub fn eval(&self, val: &[MalType], env: &Environment) -> Result<MalType> {
        debug!("MalExpr::eval: self: {:?} -- values {:?}", self, val);
        (self.inner)(val, env.clone())
    }
}

impl Debug for MalExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MalExpr")
            .field("symbol", &self.symbol)
            .field("arguments", &self.arguments)
            .finish()
    }
}

impl Display for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::List(inner) => print_collection(MalCollection::List, inner, f),
            MalType::HashMap(inner) => print_collection(MalCollection::HashMap, inner, f),
            MalType::Vector(inner) => print_collection(MalCollection::Vector, inner, f),
            MalType::String(str) => write!(f, "{}", str),
            MalType::Symbol(symbol) => write!(f, "{}", symbol),
            MalType::Number(nr) => write!(f, "{}", nr),
            MalType::Bool(b) => match b {
                true => write!(f, "true"),
                false => write!(f, "false"),
            },
            MalType::Nil => write!(f, "nil"),
            MalType::LibFn(expr) => {
                write!(f, "LibFn: {} [{}]", expr.expr.symbol, expr.captured_env)
            }
            MalType::Fn(expr) => write!(f, "Fn: {} [{}]", expr.expr, expr.captured_args),
            MalType::Bind(expr) => write!(f, "Bind: {} [{}]", expr.symbol, expr.arguments),
            MalType::BinOp(expr) => write!(f, "BinOp: {} [{}]", expr.symbol, expr.arguments),
        }
    }
}

impl Binary for MalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::List(inner) => print_collection_b(MalCollection::List, inner, f),
            MalType::HashMap(inner) => print_collection_b(MalCollection::HashMap, inner, f),
            MalType::Vector(inner) => print_collection_b(MalCollection::Vector, inner, f),
            MalType::String(str) => write!(f, "\"{}\"", escape_str(str)),
            other => write!(f, "{}", other),
        }
    }
}

fn escape_str(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\n' => "\\n".to_string(),
            '\\' => "\\\\".to_string(),
            _ => c.to_string(),
        })
        .collect::<Vec<String>>()
        .join("")
}

fn print_collection(
    collection_type: MalCollection,
    inner: &[MalType],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}", collection_type.start())?;
    for i in 0..inner.len() {
        if i > 0 {
            write!(f, " ")?;
        }

        write!(f, "{}", inner[i])?;
    }
    write!(f, "{}", collection_type.end())
}

fn print_collection_b(
    collection_type: MalCollection,
    inner: &[MalType],
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}", collection_type.start())?;
    for i in 0..inner.len() {
        if i > 0 {
            write!(f, " ")?;
        }

        write!(f, "{:b}", inner[i])?;
    }
    write!(f, "{}", collection_type.end())
}

pub enum MalCollection {
    HashMap,
    List,
    Vector,
}

impl MalCollection {
    pub fn get(value: &str) -> Self {
        match value {
            "{" => MalCollection::HashMap,
            "(" => MalCollection::List,
            "[" => MalCollection::Vector,
            _ => unreachable!(),
        }
    }

    pub const fn start(&self) -> &'static str {
        match self {
            MalCollection::HashMap => "{",
            MalCollection::List => "(",
            MalCollection::Vector => "[",
        }
    }

    pub const fn end(&self) -> &'static str {
        match self {
            MalCollection::HashMap => "}",
            MalCollection::List => ")",
            MalCollection::Vector => "]",
        }
    }

    pub fn into(self, data: Vec<MalType>) -> MalType {
        match self {
            MalCollection::HashMap => MalType::HashMap(data),
            MalCollection::List => MalType::List(data),
            MalCollection::Vector => MalType::Vector(data),
        }
    }
}

impl Add for &MalType {
    type Output = MalType;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MalType::Number(n1), MalType::Number(n2)) => MalType::Number(n1 + n2),
            _ => panic!("ADD: {:?} ---- {:?}", self, rhs),
        }
    }
}

impl Sub for &MalType {
    type Output = MalType;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MalType::Number(n1), MalType::Number(n2)) => MalType::Number(n1 - n2),
            _ => panic!("SUB: {:?} ---- {:?}", self, rhs),
        }
    }
}

impl Mul for &MalType {
    type Output = MalType;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MalType::Number(n1), MalType::Number(n2)) => MalType::Number(n1 * n2),
            _ => panic!("MUL: {:?} ---- {:?}", self, rhs),
        }
    }
}

impl Div for &MalType {
    type Output = MalType;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MalType::Number(n1), MalType::Number(n2)) => MalType::Number(n1 / n2),
            _ => panic!("DIV: {:?} ---- {:?}", self, rhs),
        }
    }
}

impl PartialEq for MalType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::List(l0), Self::List(r0)) => l0 == r0,
            (Self::HashMap(l0), Self::HashMap(r0)) => l0 == r0,
            (Self::Vector(l0), Self::Vector(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Symbol(l0), Self::Symbol(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl PartialEq<&str> for MalType {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::String(l0) => l0 == other,
            Self::Symbol(l0) => l0 == other,
            _ => false,
        }
    }
}

impl PartialOrd for &MalType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (MalType::Number(n1), MalType::Number(n2)) => n1.partial_cmp(n2),
            _ => panic!("ORD: {:?} ---- {:?}", self, other),
        }
    }
}
