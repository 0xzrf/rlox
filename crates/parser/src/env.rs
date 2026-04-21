use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpret::Value;

pub type EnvRef = Rc<RefCell<Env>>;

pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<EnvRef>,
}


impl Env {
    pub fn new(enclosing: Option<EnvRef>) -> Self {
        Self { values: HashMap::new(), enclosing }
    }

    pub fn get_owned(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.values.get(name) {
            return Some(val.clone());
        }
        self.enclosing.as_ref().and_then(|e| e.borrow().get_owned(name))
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

#[cfg(test)]
mod tests {
    use super::Env;
    use crate::interpret::Value;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn assign_updates_value_in_current_scope() {
        let mut env = Env::new(None);
        env.define("a".to_string(), Some(Value::Number(1.0)));

        env.assign("a".to_string(), Value::Number(2.0)).unwrap();
        assert_eq!(env.get_owned("a"), Some(Value::Number(2.0)));
    }

    #[test]
    fn lexical_scoping_reads_nearest_enclosing_definition() {
        let global = Rc::new(RefCell::new(Env::new(None)));
        global
            .borrow_mut()
            .define("a".to_string(), Some(Value::String("global".to_string())));

        let outer = Rc::new(RefCell::new(Env::new(Some(global.clone()))));
        outer
            .borrow_mut()
            .define("a".to_string(), Some(Value::String("outer".to_string())));

        let inner = Env::new(Some(outer.clone()));

        assert_eq!(
            inner.get_owned("a"),
            Some(Value::String("outer".to_string()))
        );
    }
}
