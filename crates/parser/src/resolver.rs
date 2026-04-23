use std::collections::HashMap;

use crate::{Interpret, Stmt};

pub struct Resolver {
    interpret: Interpret,
    scopes: Vec<HashMap<String, bool>>,
}


impl Resolver {
    pub fn new(interpre: Interpret) -> Self {
        Self { interpret, scopes: vec![] }
    }

    pub fn resolve_stmt(&mut self, stmt: Stmt) {}

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scrop(&mut self) {
        self.scopes.pop();
    }
}
