use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
};

pub struct HashQueue<K, V>
where
    K: Eq + Hash,
{
    keys: HashSet<K>,
    values: VecDeque<(K, V)>,
}

impl<K, V> HashQueue<K, V>
where
    K: PartialEq + Eq + Hash + Clone + Debug,
{
    pub fn new() -> Self {
        Self {
            keys: HashSet::new(),
            values: VecDeque::new(),
        }
    }

    pub fn pop(&mut self) -> Option<(K, V)> {
        let (k, v) = self.values.pop_front()?;
        self.keys.remove(&k);
        Some((k, v))
    }

    pub fn push(&mut self, key: K, value: V) {
        let value_new = self.keys.insert(key.clone());
        if !value_new {
            // Remove old value
            if let Some(pos) = self.values.iter().position(|(k, _)| *k == key) {
                self.values.remove(pos);
            }
        }
        self.values.push_back((key, value));
    }
}
