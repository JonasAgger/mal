use log::{debug, trace};

use crate::expr::Expressions;
use crate::types::MalType;
use std::{
    cell::{OnceCell, Ref, RefCell},
    fmt::Debug,
    rc::Rc,
    sync::Once,
};

pub struct Environment {
    inner: Rc<RefCell<InnerEnv>>,
    default_ns: Rc<Expressions>,
}

impl Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner.as_ref().borrow(), f)
    }
}

struct InnerEnv {
    outer: Option<Rc<RefCell<InnerEnv>>>,
    expressions: Expressions,
}

impl Debug for InnerEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "InnerEnv: ")?;
        if let Some(outer) = self.outer.as_ref() {
            let outer: Ref<InnerEnv> = outer.borrow();
            writeln!(f, "\tOuter: {:?}", outer)?;
        }
        writeln!(f, "\t{:?}", self.expressions)
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
            default_ns: Rc::clone(&self.default_ns),
        }
    }
}

impl Environment {
    pub fn new() -> Self {
        Self {
            inner: InnerEnv::new(Expressions::new()),
            default_ns: Rc::new(Expressions::new_default()),
        }
    }

    pub fn from(outer: Environment, keys: MalType, values: &[MalType]) -> Environment {
        let inner = InnerEnv::new(Expressions::new());
        inner.as_ref().borrow_mut().enter(outer.inner);
        let mut s = Self {
            inner,
            default_ns: outer.default_ns,
        };

        match keys {
            MalType::List(inner) => {
                for i in 0..inner.len() {
                    if inner[i] == "&" {
                        let values = values[i..].to_vec();
                        s.set(inner[i + 1].clone(), MalType::List(values));
                        break;
                    } else {
                        s.set(inner[i].clone(), values[i].clone());
                    }
                }
            }
            _ => panic!("Environment::from received non list in keys"),
        }

        s
    }

    pub fn set(&mut self, key: MalType, value: MalType) {
        trace!("Setting: {:?} -> {:?}", key, value);
        match key {
            MalType::Symbol(symbol) => self.inner.as_ref().borrow_mut().set(symbol, value),
            key => panic!("Called set with not symbol {:?}", key),
        }
    }

    pub fn get(&self, s: &str) -> Option<MalType> {
        if let Some(core_fn) = self.default_ns.get(s).cloned() {
            return Some(core_fn);
        }
        let ret = self.inner.borrow().get(s);
        trace!("getting: {:?} -> {:?}", s, ret);
        ret
    }

    pub fn enter(&mut self) {
        debug!("entering scope");
        let new_inner = InnerEnv::new(Expressions::new());
        new_inner
            .as_ref()
            .borrow_mut()
            .enter(Rc::clone(&self.inner));
        self.inner = new_inner;
    }

    pub fn exit(&mut self) {
        debug!("exiting scope");
        let outer = self.inner.as_ref().borrow_mut().exit();
        if let Some(outer) = outer {
            self.inner = outer;
        }
    }
}

impl InnerEnv {
    fn new(expressions: Expressions) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(InnerEnv {
            outer: None,
            expressions,
        }))
    }

    fn enter(&mut self, outer: Rc<RefCell<Self>>) {
        self.outer = Some(outer);
    }

    fn exit(&mut self) -> Option<Rc<RefCell<InnerEnv>>> {
        self.outer.take()
    }

    fn set(&mut self, key: String, value: MalType) {
        self.expressions.set(key, value);
    }

    fn get(&self, s: &str) -> Option<MalType> {
        if let Some(expr) = self.expressions.get(s).cloned() {
            Some(expr)
        } else {
            self.outer.as_ref().map(|x| x.borrow().get(s)).flatten()
        }
    }
}
