pub mod instructions;
pub mod interpreter;

use std::collections::HashMap;

#[derive(Clone)]
pub struct Memory {
    initial_data: Vec<i64>,
    expanded_memory: HashMap<usize, i64>,
}

impl<'a> Memory {
    pub fn new(p_data: Vec<i64>) -> Memory {
        Memory {
            initial_data: p_data,
            expanded_memory: HashMap::new(),
        }
    }

    pub fn read(&self, index: usize) -> i64 {
        if index >= self.initial_data.len() {
            if !self.expanded_memory.contains_key(&index) {
                0
            } else {
                self.expanded_memory[&index]
            }
        } else {
            self.initial_data[index]
        }
    }

    pub fn write(&mut self, index: usize, value: i64) {
        if index >= self.initial_data.len() {
            self.expanded_memory.insert(index, value);
        } else {
            self.initial_data[index] = value;
        }
    }
}
