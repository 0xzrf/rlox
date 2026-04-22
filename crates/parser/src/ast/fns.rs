use crate::Interpret;
use crate::interpret::{InterpretResult, Value};

pub struct LoxCallable;

impl LoxCallable {
    pub fn call(interpret: &mut Interpret, args: Vec<Value>) -> InterpretResult<Value> {
        todo!()
    }
}
