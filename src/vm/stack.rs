use super::{InternalError, Result, Value};

#[derive(Default, Debug, Clone)]
pub struct Stack<'b>(Vec<Value<'b>>);

impl<'b> Stack<'b> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, value: Value<'b>) -> Result<()> {
        self.0.push(value);
        Ok(())
    }

    pub fn pop(&mut self) -> Result<Value<'b>> {
        self.0.pop().ok_or_else(|| InternalError::EmptyStack.into())
    }

    pub fn pop_multiple(&mut self, count: usize) -> Result<impl Iterator<Item = Value<'b>> + '_> {
        let offset = self
            .len()
            .checked_sub(count)
            .ok_or(InternalError::EmptyStack)?;

        Ok(self.0.drain(offset..))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
