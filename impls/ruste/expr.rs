use crate::types::MalType;
use std::{collections::HashMap, fmt::Debug};

pub struct Expressions {
    expressions: HashMap<String, MalType>,
}

impl Expressions {
    pub fn new() -> Self {
        Self {
            expressions: HashMap::new(),
        }
    }

    pub fn new_default() -> Self {
        let mut s = Self::new();
        crate::core::add_functions(&mut s.expressions);
        s
    }

    pub fn set(&mut self, k: String, v: MalType) {
        self.expressions.insert(k, v);
    }

    pub fn get(&self, k: &str) -> Option<&MalType> {
        self.expressions.get(k)
    }
}

impl Debug for Expressions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Expressions:")?;
        for (key, val) in self.expressions.iter() {
            writeln!(f, "\t{:?} -> {:?}", key, val)?;
        }

        Ok(())
    }
}
