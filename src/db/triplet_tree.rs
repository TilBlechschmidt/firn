use super::length_prefixed_concatenate_merge;
use sled::{IVec, Tree};

pub struct TripletTree {
    tree: Tree,
}

impl TripletTree {
    /// Constructs a new TripletTree from a Tree. Cheap to use.
    /// TripletTree operations will panic if the merge operator has not been set.
    pub fn new(tree: Tree) -> Self {
        tree.set_merge_operator(length_prefixed_concatenate_merge);
        Self { tree }
    }

    /// Appends the value to the given key, retaining old values if present
    pub fn append(
        &self,
        key1: impl AsRef<[u8]>,
        key2: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> sled::Result<()> {
        self.tree
            .merge(TripletTree::build_key(key1, key2), value)
            .map(|_| ())
    }

    /// Retrieves a set of values, returns an iterator which may be empty
    pub fn get(&self, key1: impl AsRef<[u8]>, key2: impl AsRef<[u8]>) -> sled::Result<ValueSet> {
        Ok(self
            .tree
            .get(TripletTree::build_key(key1, key2))?
            .map(ValueSet)
            .unwrap_or_default())
    }

    /// Deletes _all_ values for a given key, returning the old value if it existed
    pub fn remove(&self, key1: impl AsRef<[u8]>, key2: impl AsRef<[u8]>) -> sled::Result<()> {
        self.tree.remove(TripletTree::build_key(key1, key2))?;
        Ok(())
    }

    pub fn clear(&self) -> sled::Result<()> {
        self.tree.clear()
    }

    /// Searches the tree for entries where key2 is a prefix of the entries key2. Does _not_ support key1 prefix search!
    pub fn scan_key2_prefix(
        &self,
        key1: impl AsRef<[u8]>,
        key2: impl AsRef<[u8]>,
    ) -> impl Iterator<Item = sled::Result<(IVec, IVec, ValueSet)>> {
        self.tree
            .scan_prefix(TripletTree::build_key(key1, key2))
            .map(|result| match result {
                Err(e) => Err(e),
                Ok((key, value)) => {
                    if let Some((key1, key2)) = split_key(key) {
                        Ok((key1, key2, ValueSet(value)))
                    } else {
                        Err(sled::Error::ReportableBug(
                            "encountered corrupted key".into(),
                        ))
                    }
                }
            })
    }

    pub fn scan(&self) -> impl Iterator<Item = sled::Result<(IVec, IVec, ValueSet)>> {
        self.scan_key2_prefix([], [])
    }

    fn build_key(key1: impl AsRef<[u8]>, key2: impl AsRef<[u8]>) -> Vec<u8> {
        let len_key1 = key1.as_ref().len();
        let len_key2 = key2.as_ref().len();

        if len_key1 + len_key2 == 0 {
            return Vec::new();
        }

        let mut key = Vec::with_capacity(4 + len_key1 + len_key2);
        leb128::write::unsigned(&mut key, len_key1 as u64).expect("failed to write to vector");
        key.extend_from_slice(key1.as_ref());
        key.extend_from_slice(key2.as_ref());

        key
    }
}

impl From<Tree> for TripletTree {
    fn from(tree: Tree) -> Self {
        Self::new(tree)
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct ValueSet(IVec);

impl Iterator for ValueSet {
    type Item = IVec;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = self.0.as_ref();
        let original_len = buffer.len();
        let value_len = leb128::read::unsigned(&mut buffer).ok()? as usize;
        let prefix_len = original_len - buffer.len();
        let value = self.0.subslice(prefix_len, value_len);

        self.0 = self.0.subslice(
            prefix_len + value_len,
            original_len - prefix_len - value_len,
        );

        Some(value)
    }
}

fn split_key(vec: IVec) -> Option<(IVec, IVec)> {
    let mut buffer = vec.as_ref();
    let original_len = buffer.len();
    let key1_len = leb128::read::unsigned(&mut buffer).ok()? as usize;
    let prefix_len = original_len - buffer.len();

    let key1 = vec.subslice(prefix_len, key1_len);
    let key2 = vec.subslice(prefix_len + key1_len, original_len - prefix_len - key1_len);

    Some((key1, key2))
}

#[cfg(test)]
mod does {
    use super::*;

    fn create_tree() -> sled::Result<TripletTree> {
        let db = sled::Config::new().temporary(true).open()?;
        let tree = db.open_tree("test")?;
        Ok(TripletTree::new(tree))
    }

    #[test]
    fn merge_values() -> sled::Result<()> {
        let tree = create_tree()?;
        tree.append([1], [2], "test1")?;
        tree.append([1], [2], "test2")?;

        let mut values = tree.get([1], [2])?;
        assert_eq!(values.next().unwrap(), "test1");
        assert_eq!(values.next().unwrap(), "test2");
        assert_eq!(values.next(), None);

        Ok(())
    }

    #[test]
    fn separate_keys() -> sled::Result<()> {
        let tree = create_tree()?;
        tree.append([1], [2, 3], "test1")?;
        tree.append([1, 2], [3], "test2")?;

        assert_eq!(tree.get([1], [2, 3])?.next().unwrap(), "test1");
        assert_eq!(tree.get([1, 2], [3])?.next().unwrap(), "test2");

        Ok(())
    }

    #[test]
    fn remove_values() -> sled::Result<()> {
        let tree = create_tree()?;

        tree.append([1], [2], "test1")?;
        assert_eq!(tree.get([1], [2])?.next().unwrap(), "test1");

        tree.remove([1], [2])?;
        assert_eq!(tree.get([1], [2])?.next(), None);

        Ok(())
    }

    #[test]
    fn scan_whole_tree() -> sled::Result<()> {
        let tree = create_tree()?;
        tree.append([1], [2, 3], "test1")?;
        tree.append([1, 2], [3], "test2")?;

        let mut scanner = tree.scan_key2_prefix([], []);
        let mut entry1 = scanner.next().unwrap().unwrap();
        let mut entry2 = scanner.next().unwrap().unwrap();

        assert_eq!(scanner.next(), None);
        assert_eq!(entry1.0, [1]);
        assert_eq!(entry1.1, [2, 3]);
        assert_eq!(entry2.0, [1, 2]);
        assert_eq!(entry2.1, [3]);
        assert_eq!(entry1.2.next().unwrap(), "test1");
        assert_eq!(entry2.2.next().unwrap(), "test2");

        Ok(())
    }
}
