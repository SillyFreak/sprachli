use std::collections::HashMap;
use std::ops;

use super::Value;

#[derive(Default, Debug, Clone)]
pub struct Environment {
    bindings: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.bindings.insert(name, value)
    }

    pub fn get<Q: ?Sized>(&self, name: &Q) -> Option<&Value>
    where
        String: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        self.bindings.get(name)
    }
}

impl<Q: ?Sized> ops::Index<&Q> for Environment
where
    String: std::borrow::Borrow<Q>,
    Q: Eq + std::hash::Hash,
{
    type Output = Value;

    fn index(&self, name: &Q) -> &Self::Output {
        &self.bindings[name]
    }
}
