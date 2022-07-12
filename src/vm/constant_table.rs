use std::collections::HashMap;

use super::Value;

#[derive(Debug, Clone)]
pub struct ConstantTable {
    table: Vec<Value>,
}

impl ConstantTable {
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.table.get(index)
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
