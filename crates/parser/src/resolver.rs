use std::collections::HashMap;

use interpreter_types::Token;

use crate::errors::CompileTimeError;
use crate::{Expr, Interpret, Stmt};

pub struct Resolver<'a> {
    interpret: &'a mut Interpret,
    scopes: Vec<HashMap<String, bool>>,
    current_fn: FunctionType,
}

pub type ResolverResult<T> = Result<T, CompileTimeError>;
#[derive(Copy, PartialEq, Clone)]
pub enum FunctionType {
    None,
    Function,
}

impl<'a> Resolver<'a> {
    pub fn new(interpret: &'a mut Interpret) -> Self {
        Self {
            interpret,
            scopes: vec![],
            current_fn: FunctionType::None,
        }
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> ResolverResult<()> {
        match stmt {
            Stmt::Block { stmts } => {
                self.begin_scope();
                for stmt in stmts {
                    self.resolve_stmt(stmt)?;
                }
                self.end_scrop();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name)?;
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name);
            }
            Stmt::Function { name, params: _, body: _ } => {
                self.define(name);
                self.declare(name)?;

                self.resolve_fn(stmt, FunctionType::Function)?;
            }
            Stmt::Expression { expr } => {
                self.resolve_expr(&expr)?;
            }
            Stmt::IfStmt {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch)?;
                }
            }
            Stmt::Print { expr } => {
                self.resolve_expr(expr)?;
            }
            Stmt::Return { keyword, value } => {
                if self.current_fn == FunctionType::None {
                    return Err(CompileTimeError {
                        token: keyword.clone(),
                        message: "Cannot return outside a function",
                    });
                }

                if let Some(return_val) = value {
                    self.resolve_expr(return_val)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
        }
        Ok(())
    }

    fn resolve_expr(&mut self, expr: &Expr) -> ResolverResult<()> {
        match expr {
            Expr::Variable { name } => {
                if let Some(current_scope) = self.get_current_scope_borrow()
                    && matches!(current_scope.get(&name.lexeme), Some(false))
                {
                    return Err(CompileTimeError {
                        token: name.clone(),
                        message: "Cannot read local variable in its own initializer",
                    });
                }

                self.resolve_local(expr, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expr, name);
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let _ = operator;
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Call {
                callee,
                paren,
                args,
            } => {
                let _ = paren;
                self.resolve_expr(callee)?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Grouping { expression } => {
                self.resolve_expr(expression)?;
            }
            Expr::Literal { value: _ } => {}
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let _ = operator;
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Unary { operator, right } => {
                let _ = operator;
                self.resolve_expr(right)?;
            }
        }

        Ok(())
    }

    fn resolve_fn(&mut self, stmt: &Stmt, fn_type: FunctionType) -> ResolverResult<()> {
        let Stmt::Function { name, params, body } = stmt else {
            unreachable!()
        };
        let _ = name;
        let enclosing_fn = self.current_fn;
        self.current_fn = fn_type;
        self.begin_scope();

        for param in params {
            self.declare(param)?;
            self.define(param);
        }

        self.resolve_stmts(body)?;

        self.end_scrop();

        self.current_fn = enclosing_fn;
        Ok(())
    }

    fn resolve_stmts(&mut self, stmts: &[Stmt]) -> ResolverResult<()> {
        for stmt in stmts {
            self.resolve_stmt(stmt)?;
        }
        Ok(())
    }

    fn resolve_local(&mut self, expr: &Expr, name: &Token) {
        for (ix, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpret.resolve(expr, ix);
                return;
            }
        }
    }

    fn declare(&mut self, name: &Token) -> ResolverResult<()> {
        if self.is_scope_empty() {
            return Ok(());
        };

        if let Some(current_scope) = self.get_current_scope_mut() {
            let name_clone = name.lexeme.clone();
            if current_scope.get(&name_clone).is_some() {
                return Err(CompileTimeError {
                    token: name.clone(),
                    message: "Cannot redeclare a variable",
                });
            }
            current_scope.insert(name_clone, false);
        }
        Ok(())
    }

    fn define(&mut self, name: &Token) {
        if self.is_scope_empty() {
            return;
        };

        if let Some(current_scope) = self.get_current_scope_mut()
            && let Some(ident_mut) = current_scope.get_mut(&name.lexeme)
        {
            *ident_mut = true;
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
