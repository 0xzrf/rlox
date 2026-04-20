use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::interpret::Value;

pub type EnvRef = Rc<RefCell<Env>>;

pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<EnvRef>,
}


impl Env {
    pub fn new(enclosing: Option<EnvRef>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }

    pub fn get_owned(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.values.get(name) {
            return Some(val.clone());
        }
        self.enclosing
            .as_ref()
            .and_then(|e| e.borrow().get_owned(name))
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        if let Some(inner_val) = value {
            self.values.insert(name, inner_val);
        } else {
            self.values.insert(name, Value::Nil);
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), String> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }

        Err(format!("Undefined variable '{}'.", name))

    }
}
