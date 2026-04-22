use std::time::{SystemTime, UNIX_EPOCH};

use crate::Interpret;
use crate::interpret::{InterpretResult, Value};

pub trait LoxCallable {
    fn call(&mut self, interpreter: &mut Interpret, args: Vec<Value>) -> InterpretResult<Value>;

    fn arity(&self) -> usize;
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

    fn call(&mut self, interpreter: &mut Interpret, _args: Vec<Value>) -> InterpretResult<Value> {
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
