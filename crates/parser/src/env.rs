use std::collections::HashMap;

use crate::interpret::{RuntimeError, Value};

pub struct Env {
    values: HashMap<String, Option<Value>>,
}


impl Env {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn get_var(&self, name: &str) -> Option<&Option<Value>> {
        self.values.get(name)
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.values.insert(name, value);
    }
}
