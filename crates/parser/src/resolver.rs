use std::collections::HashMap;

use interpreter_types::Token;

use crate::{Expr, Interpret, Stmt};

pub struct Resolver {
    interpret: Interpret,
    scopes: Vec<HashMap<String, bool>>,
}


impl Resolver {
    pub fn new(interpre: Interpret) -> Self {
        Self { interpret, scopes: vec![] }
    }

    pub fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block { stmts } => {}
            Stmt::Var { name, initializer } => {
                self.declare(&name);
                if let Some(ref init) = initializer {
                    self.resolve_expr(expr);
                }
                self.define(&name);
            }
            _ => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {}

    fn declare(&mut self, name: &Token) {
        if self.is_scope_empty() {
            return;
        };

        if let Some(current_scope) = self.get_current_scope() {
            current_scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if self.is_scope_empty() {
            return;
        };

        if let Some(current_scope) = self.get_current_scope()
            && let Some(mut ident_mut) = current_scope.get_mut(&name.lexeme)
        {
            ident_mut = &mut true;
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }

    fn end_scrop(&mut self) {
        self.scopes.pop();
    }

    fn is_scope_empty(&self) -> bool {
        self.scopes.is_empty()
    }

    fn get_current_scope(&mut self) -> Option<&mut HashMap<String, bool>> {
        self.scopes.last_mut()
    }
}
