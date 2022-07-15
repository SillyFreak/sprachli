use crate::vm::{Result, Value};

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

    pub fn pop(&mut self) -> Value {
        self.0.pop().expect("empty stack")
    }

    pub fn pop_call(&mut self, arity: usize) -> Result<(Value, Vec<Value>)> {
        let offset = self.len() - arity - 1;

        let function = self.get(offset)
            .expect("stack frame without function")
            .as_function()?;

        function.check_arity(arity)?;

        let parameters = self.0.drain((offset + 1)..).collect::<Vec<_>>();
        let function = self.pop();

        Ok((function, parameters))
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
