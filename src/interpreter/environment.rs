use std::collections::HashMap;

use super::Value;

#[derive(Default, Debug, Clone)]
pub struct Environment<'a> {
    parent: Option<&'a Environment<'a>>,
    bindings: HashMap<String, Value>,
}

impl<'a> Environment<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: &'a Environment<'a>) -> Self {
        Self {
            parent: Some(parent),
            bindings: Default::default(),
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Option<Value> {
        self.bindings.insert(name, value)
    }

    pub fn get<Q: ?Sized>(&self, name: &Q) -> Option<&Value>
    where
        String: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        if let Some(value) = self.bindings.get(name) {
            return Some(value);
        }

        self.parent.and_then(|env| env.get(name))
    }
}
