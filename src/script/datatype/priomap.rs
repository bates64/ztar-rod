use std::hash::Hash;
use std::collections::{VecDeque, HashMap};

/// A priority-queue map. Operates on layers; values are inserted into the
/// highest-priority layer, and lookups return the highest-priority match.
pub struct PrioMap<K: Hash + Eq, V> {
    layers: VecDeque<HashMap<K, V>>,
}

impl<K: Hash + Eq, V> PrioMap<K, V> {
    /// Creates a new PrioMap with a single layer.
    pub fn new() -> PrioMap<K, V> {
        let mut pm = PrioMap {
            layers: VecDeque::new(),
        };

        pm.push();
        pm
    }

    /// Creates a new higher-priority layer.
    pub fn push(&mut self) {
        self.layers.push_front(HashMap::new());
    }

    /// Removes the highest-priority layer and returns it, if any.
    pub fn pop(&mut self) -> Option<HashMap<K, V>> {
        self.layers.pop_front()
    }

    /// Inserts a value to the highest-priority layer, shadowing values below
    /// that use the same key. If the layer already contains the key, its
    /// previous value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.layers[0].insert(key, value)
    }

    /// Looks-up the value of the given key. Higher-priority layers shadow the
    /// values of lower ones if they have the same key.
    pub fn lookup(&self, key: K) -> Option<&V> {
        for layer in self.layers.iter() {
            if let Some(value) = layer.get(&key) {
                return Some(value);
            }
        }

        None
    }
}
