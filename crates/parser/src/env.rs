use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpret::Value;

pub type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone)]
pub struct Env {
    values: HashMap<String, Value>,
    enclosing: Option<EnvRef>,
}


impl Env {
    pub fn new(enclosing: Option<EnvRef>) -> Self {
        Self { values: HashMap::new(), enclosing }
    }

    pub fn enclosing(&self) -> Option<EnvRef> {
        self.enclosing.clone()
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

    // NOTE: we intentionally avoid `get_at` / `assign_at` helpers here for now.
    // The interpreter walks the `EnvRef` chain directly to avoid cloning `Env`s
    // or relying on `Rc::get_mut()` (which usually fails once environments are shared).
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::Env;
    use crate::interpret::Value;

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

        assert_eq!(inner.get_owned("a"), Some(Value::String("outer".to_string())));
    }

    #[test]
    fn assign_updates_value_in_enclosing_scope() {
        let global = Rc::new(RefCell::new(Env::new(None)));
        global.borrow_mut().define("a".to_string(), Some(Value::Number(1.0)));

        let mut inner = Env::new(Some(global.clone()));
        inner.assign("a".to_string(), Value::Number(3.0)).unwrap();

        assert_eq!(global.borrow().get_owned("a"), Some(Value::Number(3.0)));
        assert_eq!(inner.get_owned("a"), Some(Value::Number(3.0)));
    }

    #[test]
    fn assign_errors_on_undefined_variable() {
        let mut env = Env::new(None);
        let err = env
            .assign("missing".to_string(), Value::Nil)
            .expect_err("expected assignment to undefined variable to error");
        assert!(err.contains("Undefined variable 'missing'"), "got: {err}");
    }

    #[test]
    fn define_without_value_defaults_to_nil() {
        let mut env = Env::new(None);
        env.define("a".to_string(), None);
        assert_eq!(env.get_owned("a"), Some(Value::Nil));
    }

    #[test]
    fn get_owned_returns_none_for_missing_variable() {
        let global = Rc::new(RefCell::new(Env::new(None)));
        let inner = Env::new(Some(global));
        assert_eq!(inner.get_owned("missing"), None);
    }

    #[test]
    fn inner_define_shadows_global_without_mutating_global() {
        let global = Rc::new(RefCell::new(Env::new(None)));
        global.borrow_mut().define("a".to_string(), Some(Value::Number(1.0)));

        let mut inner = Env::new(Some(global.clone()));
        inner.define("a".to_string(), Some(Value::Number(2.0)));

        assert_eq!(inner.get_owned("a"), Some(Value::Number(2.0)));
        assert_eq!(global.borrow().get_owned("a"), Some(Value::Number(1.0)));
    }

    #[test]
    fn assign_updates_nearest_defined_scope() {
        let global = Rc::new(RefCell::new(Env::new(None)));
        global.borrow_mut().define("a".to_string(), Some(Value::Number(1.0)));

        let middle = Rc::new(RefCell::new(Env::new(Some(global.clone()))));
        middle.borrow_mut().define("a".to_string(), Some(Value::Number(2.0)));

        let mut inner = Env::new(Some(middle.clone()));
        inner.assign("a".to_string(), Value::Number(3.0)).unwrap();

        assert_eq!(global.borrow().get_owned("a"), Some(Value::Number(1.0)));
        assert_eq!(middle.borrow().get_owned("a"), Some(Value::Number(3.0)));
        assert_eq!(inner.get_owned("a"), Some(Value::Number(3.0)));
    }
}
