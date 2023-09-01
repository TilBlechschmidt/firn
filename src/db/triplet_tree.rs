use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct TripletTree<K1: Hash + Eq, K2: Hash + Eq, V>(HashMap<(K1, K2), Vec<V>>);

impl<K1: Hash + Eq + Clone, K2: Hash + Eq + Clone, V> Default for TripletTree<K1, K2, V> {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl<K1: Hash + Eq + Clone, K2: Hash + Eq + Clone, V: Clone> TripletTree<K1, K2, V> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Appends the value to the given key, retaining old values if present
    pub fn append(&mut self, key1: K1, key2: K2, value: V) {
        self.0
            .entry((key1, key2))
            .and_modify(|set| set.push(value.clone()))
            .or_insert(vec![value]);
    }

    /// Retrieves all attribute + value combinations for a given first key
    pub fn get<'k>(&'k self, key1: &'k K1) -> impl Iterator<Item = (&K2, &Vec<V>)> + 'k {
        self.0
            .iter()
            .filter(move |((k1, _), _)| key1 == k1)
            .map(|((_, k2), v)| (k2, v))
    }

    /// Retrieves a set of values, returns an iterator which may be empty
    pub fn values(&self, key1: &K1, key2: &K2) -> impl Iterator<Item = &V> {
        self.0
            .get(&((*key1).clone(), (*key2).clone()))
            .map(|set| set.iter())
            .unwrap_or_default()
    }

    /// Deletes _all_ values for a given key, returning the old value if it existed
    pub fn remove(&mut self, key1: &K1, key2: &K2) {
        self.0.remove(&((*key1).clone(), (*key2).clone()));
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn scan(&self) -> impl Iterator<Item = (&K1, &K2, &Vec<V>)> {
        self.0.iter().map(|(k, v)| (&k.0, &k.1, v))
    }
}

#[cfg(test)]
mod does {
    use super::*;

    fn create_tree<A: Hash + Eq + Clone, B: Hash + Eq + Clone, C>() -> TripletTree<A, B, C> {
        TripletTree::default()
    }

    // #[test]
    // fn merge_values() {
    //     let tree = create_tree();
    //     tree.append([1], [2], "test1");
    //     tree.append([1], [2], "test2");

    //     let mut values = tree.get(&[1], &[2]);
    //     assert_eq!(values.next().unwrap(), "test1");
    //     assert_eq!(values.next().unwrap(), "test2");
    //     assert_eq!(values.next(), None);
    // }

    // #[test]
    // fn separate_keys() {
    //     let tree = create_tree();
    //     tree.append("a", "b", "test1");
    //     tree.append("c", "d", "test2");

    //     assert_eq!(tree.get("a", "b").next().unwrap(), "test1");
    //     assert_eq!(tree.get("c", "d").next().unwrap(), "test2");
    // }

    // #[test]
    // fn remove_values() {
    //     let tree = create_tree();

    //     tree.append([1], [2], "test1");
    //     assert_eq!(tree.get([1], [2]).next().unwrap(), "test1");

    //     tree.remove([1], [2])?;
    //     assert_eq!(tree.get([1], [2]).next(), None);
    // }

    // #[test]
    // fn scan_whole_tree() {
    //     let tree = create_tree();
    //     tree.append("a", "b", "test1");
    //     tree.append("c", "d", "test2");

    //     let mut scanner = tree.scan_key2_prefix([], []);
    //     let mut entry1 = scanner.next().unwrap().unwrap();
    //     let mut entry2 = scanner.next().unwrap().unwrap();

    //     assert_eq!(scanner.next(), None);
    //     assert_eq!(entry1.0, [1]);
    //     assert_eq!(entry1.1, [2, 3]);
    //     assert_eq!(entry2.0, [1, 2]);
    //     assert_eq!(entry2.1, [3]);
    //     assert_eq!(entry1.2.next().unwrap(), "test1");
    //     assert_eq!(entry2.2.next().unwrap(), "test2");
    // }
}
