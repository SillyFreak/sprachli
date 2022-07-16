use std::collections::HashMap;

use super::{InternalError, Result, Value};

#[derive(Debug, Clone)]
pub enum Environment<'a> {
    Root(&'a HashMap<String, Value>),
    Child {
        parent: &'a Environment<'a>,
        bindings: HashMap<String, Value>,
    }
}

impl<'a> Environment<'a> {
    pub fn root(bindings: &'a HashMap<String, Value>) -> Self {
        Self::Root(bindings)
    }

    pub fn child(parent: &'a Environment<'a>) -> Self {
        Self::Child {
            parent,
            bindings: Default::default(),
        }
    }

    pub fn set(&mut self, name: String, value: Value) -> Result<Option<Value>> {
        use Environment::*;

        match self {
            Root(_) => {
                Err(InternalError::WriteGlobalScope.into())
            }
            Child { bindings, .. } => {
                Ok(bindings.insert(name, value))
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, name: &Q) -> Option<&Value>
    where
        String: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash,
    {
        use Environment::*;

        let (parent, bindings) = match *self {
            Root(bindings) => (None, bindings),
            Child { parent, ref bindings } => (Some(parent), bindings),
        };

        bindings.get(name).or_else(|| parent.and_then(|env| env.get(name)))
    }
}
