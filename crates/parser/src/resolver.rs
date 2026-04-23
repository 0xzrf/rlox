use std::collections::HashMap;

use interpreter_types::Token;

use crate::errors::CompileTimeError;
use crate::{Expr, Interpret, Stmt};

pub struct Resolver {
    interpret: Interpret,
    scopes: Vec<HashMap<String, bool>>,
}

pub type ResolverResult<T> = Result<T, CompileTimeError>;


impl Resolver {
    pub fn new(interpret: Interpret) -> Self {
        Self { interpret, scopes: vec![] }
    }

    pub fn resolve_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Block { stmts } => {}
            Stmt::Var { name, initializer } => {
                self.declare(&name);
                if let Some(ref init) = initializer {
                    self.resolve_expr(init);
                }
                self.define(&name);
            }
            _ => {}
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult<()> {
        match expr {
            Expr::Variable { name } => {
                if let Some(current_scope) = self.get_current_scope_borrow()
                    && let Some(ident_name) = current_scope.get(&name.lexeme)
                {
                    return Err(CompileTimeError {
                        token: name.clone,
                        message: "Cannot assign a variable to itself",
                    });
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) -> ResolverResult<()> {
        for (ix, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                // TODO: self.interpret.resolv
                return Ok(());
            }
        }
        Err(CompileTimeError {
            token: Default::default(),
            message: "Assignment to undeclared value",
        })
    }

    fn declare(&mut self, name: &Token) {
        if self.is_scope_empty() {
            return;
        };

        if let Some(current_scope) = self.get_current_scope_mut() {
            current_scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if self.is_scope_empty() {
            return;
        };

        if let Some(current_scope) = self.get_current_scope_borrow()
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

    fn get_current_scope_mut(&mut self) -> Option<&mut HashMap<String, bool>> {
        self.scopes.last_mut()
    }

    fn get_current_scope_borrow(&mut self) -> Option<&HashMap<String, bool>> {
        self.scopes.last()
    }
}
