use std::collections::HashMap;

use crate::interpret::{RuntimeError, Value};

pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<Env>,
}


impl Env {
    pub fn new(enclosing: Option<Env>) -> Self {
        Self { values: HashMap::new(), enclosing }
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        match (self.values.get(name), self.enclosing) {
            (Some(val), _) => return Some(val),
            (None, Some(enclosing)) => return enclosing.get_var(name),
            _ => None,
        }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        if let Some(inner_val) = value {
            self.values.insert(name, inner_val);
        } else {
            self.values.insert(name, Value::Nil);
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeError> {
        match (self.values.insert(name, value), self.enclosing) {
            (Some(val), _) => {
                if self.values.insert(name, value).is_none() {
                    return Err(RuntimeError {
                        token: Default::default(),
                        message: format!("Undefined variable '{}'.", name),
                    });
                }
            }
            (None, Some(mut enclosing)) => {
                enclosing.assign(name, value)?;
            }
            _ => {
                return Err(RuntimeError {
                    token: Default::default(),
                    message: format!("Undefined variable '{}'.", name),
                });
            }
        }

        Ok(())
    }
}
