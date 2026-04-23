use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{Stmt, StmtEvalType};
use crate::Interpret;
use crate::env::Env;
use crate::interpret::{InterpretResult, Value};

pub trait LoxCallable {
    fn call(&self, interpreter: &mut Interpret, args: Vec<Value>) -> InterpretResult<Value>;

    fn arity(&self) -> usize;
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoxFunction {
    pub declaration: Stmt,
}

impl LoxFunction {
    pub fn new(declaration: Stmt) -> Self {
        let Stmt::Function { .. } = declaration else {
            panic!("Expected a function statement for lox function"); // this is necessary to avoid calling this on an invalid statement
        };

        Self { declaration }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        let Stmt::Function { params, .. } = &self.declaration else {
            panic!("Expected a function statement for lox function"); // this is necessary to avoid calling this on an invalid statement
        };

        params.len()
    }

    fn call(&self, interpreter: &mut Interpret, args: Vec<Value>) -> InterpretResult<Value> {
        let env = Rc::new(RefCell::new(Env::new(Some(interpreter.env.clone())))); // ASK: should this really be a global?

        let Stmt::Function { params, body, .. } = &self.declaration else {
            panic!("Expected a function statement for lox function"); // this is necessary to avoid calling this on an invalid statement
        };

        if args.len() != params.len() {
            panic!("Expected function args to be equal to args len");
        }

        for (param, arg) in params.iter().zip(args) {
            env.borrow_mut().define(param.lexeme.clone(), Some(arg));
        }

        if let StmtEvalType::Return(return_val) = interpreter.execute_block_with_env(&body, env)? {
            return Ok(return_val);
        }

        Ok(Value::Nil)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NativeFn {
    Clock,
}

impl LoxCallable for NativeFn {
    fn arity(&self) -> usize {
        match self {
            NativeFn::Clock => 0,
        }
    }

    fn call(&self, interpreter: &mut Interpret, _args: Vec<Value>) -> InterpretResult<Value> {
        match self {
            NativeFn::Clock => {
                let since_epoch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system time should be after unix epoch");
                Ok(Value::Number(since_epoch.as_secs_f64()))
            }
        }
    }
}

impl std::fmt::Display for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeFn::Clock => write!(f, "<native fn>"),
        }
    }
}
