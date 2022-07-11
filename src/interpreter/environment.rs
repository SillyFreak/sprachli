use std::collections::HashMap;

use super::Value;

#[derive(Default, Debug, Clone)]
pub struct Environment<'input, 'a> {
    parent: Option<&'a Environment<'input, 'a>>,
    bindings: HashMap<String, Value<'input>>,
}

impl<'input, 'a> Environment<'input, 'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_parent(parent: &'a Environment<'input, 'a>) -> Self {
        Self {
            parent: Some(parent),
            bindings: Default::default(),
        }
    }

    pub fn set(&mut self, name: String, value: Value<'input>) -> Option<Value> {
        self.bindings.insert(name, value)
    }

    pub fn get<Q: ?Sized>(&self, name: &Q) -> Option<&Value<'input>>
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
