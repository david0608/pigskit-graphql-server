use std::{
    cmp::Eq,
    hash::Hash,
    collections::HashMap,
};

pub struct Dict<K, V> {
    values: Vec<V>,
    map: HashMap<K, usize>,
}

impl<K: Hash + Eq, V> Dict<K, V> {
    pub fn new() -> Self {
        Dict {
            values: Vec::new(),
            map: HashMap::new(),
        }
    }

    pub fn values(self) -> Vec<V> {
        self.values
    }

    pub fn ref_values(&self) -> &Vec<V> {
        &self.values
    }

    pub fn ref_mut_value(&mut self, key: K) -> Option<&mut V> {
        if let Some(&idx) = self.map.get(&key) {
            Some(&mut self.values[idx])
        } else {
            None
        }
    }

    pub fn insert_uncheck(&mut self, key: K, value: V) -> &mut V {
        let idx = self.values.len();
        assert_eq!(self.map.insert(key, idx), None);
        self.values.push(value);
        &mut self.values[idx]
    }
}