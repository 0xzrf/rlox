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

    pub fn get_at(&self, distance: usize, name: &str) -> Option<Value> {
        self.ancestor(distance).borrow().get_owned(name)
    }

    pub fn assigne_at(&mut self, distance: usize, name: String, value: Value) {
        self.ancestor_mut(distance).assign(name, value);
    }

    pub fn ancestor_mut(&mut self, distance: usize) -> &mut Env {
        let mut env: *mut Env = self;
        for _ in 0..distance {
            // NOTE: This requires the enclosing `Rc<RefCell<Env>>` to be uniquely owned
            // at the time of mutation, otherwise we can't soundly produce an `&mut Env`.
            //
            // If you want this to work with shared environments (the common case),
            // change the API to return an `EnvRef` (or `RefMut<Env>`) instead of `&mut Env`.
            env = unsafe {
                let enclosing_rc: &mut Rc<RefCell<Env>> = (*env)
                    .enclosing
                    .as_mut()
                    .expect("missing enclosing environment while walking ancestors");

                let enclosing_cell: &mut RefCell<Env> = Rc::get_mut(enclosing_rc)
                    .expect("cannot get mutable ancestor through shared Rc; use EnvRef-based API");

                enclosing_cell.get_mut()
            };
        }
        unsafe { &mut *env }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Self>> {
        let mut env = Rc::new(RefCell::new(self.clone()));
        for _ in 0..distance {
            let enclosing = self.enclosing.clone().unwrap();
            env = enclosing;
        }

        env
    }
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
