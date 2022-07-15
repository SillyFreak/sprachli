use crate::vm::{InternalError, Result, Value};

#[derive(Default, Debug, Clone)]
pub struct Stack(Vec<Value>);

impl Stack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, value: Value) -> Result<()> {
        self.0.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value> {
        self.0.pop().ok_or_else(|| InternalError::EmptyStack.into())
    }

    pub fn pop_call(&mut self, arity: usize) -> Result<(Value, Vec<Value>)> {
        let offset = self.len().checked_sub(arity + 1)
            .ok_or(InternalError::EmptyStack)?;

        let function = self.0[offset].as_function()?;

        function.check_arity(arity)?;

        let parameters = self.0.drain((offset + 1)..).collect::<Vec<_>>();
        let function = self.pop()?;

        Ok((function, parameters))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
