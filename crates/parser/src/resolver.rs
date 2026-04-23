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
        match &stmt {
            Stmt::Block { stmts } => {
                self.begin_scope();
                // TODO
                self.end_scrop();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init);
                }
                self.define(name);
            }
            Stmt::Function { name, params, body } => {
                self.define(name);
                self.declare(name);

                self.resolve_fn(stmt);
            }
            Stmt::Expression { expr } => {
                self.resolve_expr(&expr);
            }
            Stmt::IfStmt { condition, then_branch, else_branch } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*then_branch.clone());
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(*else_branch.clone());
                }
            }
            Stmt::Print { expr } => {
                self.resolve_expr(expr);
            }
            Stmt::Return { keyword, value } => {
                if let Some(return_val) = value {
                    self.resolve_expr(return_val);
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition);
                self.resolve_stmt(*body.clone());
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult<()> {
        match expr {
            Expr::Variable { name } => {
                if let Some(current_scope) = self.get_current_scope_borrow()
                    && let Some(ident_name) = current_scope.get(&name.lexeme)
                {
                    return Err(CompileTimeError {
                        token: name.clone(),
                        message: "Cannot assign a variable to itself",
                    });
                }

                self.resolve_local(expr, name);

                Ok(())
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value);
                self.resolve_local(expr, name);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn resolve_fn(&mut self, stmt: Stmt) {
        let Stmt::Function { name, params, body } = stmt else { unreachable!() };

        self.begin_scope();

        for param in &params {
            self.declare(&param);
            self.define(&param);
        }

        self.resolve_stmts(&body);

        self.end_scrop();
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt]) {}

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (ix, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                // TODO: self.interpret.resolve
                return;
            }
        }
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

        if let Some(current_scope) = self.get_current_scope_mut()
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
