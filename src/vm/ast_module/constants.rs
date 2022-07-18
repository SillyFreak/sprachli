use std::collections::HashMap;

use super::{InternalError, Result, Value};

#[derive(Debug, Clone)]
pub struct ConstantTable {
    pub table: Vec<Value>,
}

impl ConstantTable {
    pub fn get(&self, index: usize) -> Result<&Value> {
        self.table
            .get(index)
            .ok_or_else(|| InternalError::InvalidConstant(index, self.table.len()).into())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ConstantTableBuilder {
    table: Vec<Value>,
    map: HashMap<Value, usize>,
}

impl ConstantTableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_table(self) -> ConstantTable {
        let table = self.table;
        ConstantTable { table }
    }

    pub fn insert(&mut self, value: Value) -> usize {
        if let Some(&index) = self.map.get(&value) {
            index
        } else {
            let index = self.table.len();
            self.table.push(value.clone());
            self.map.insert(value, index);
            index
        }
    }
}
