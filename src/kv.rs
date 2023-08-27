use serde::{de::DeserializeOwned, Serialize};
use sled::{Db, Tree};
use std::{error::Error, marker::PhantomData};

pub struct KeyValueStore<T> {
    tree: Tree,
    phantom: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> KeyValueStoreTree<T> {
    pub fn new<T>(tree: Tree) -> Self {
        Self {
            tree,
            phantom: PhantomData,
        }
    }

    pub fn insert(&self, key: impl AsRef<[u8]>, value: &T) -> Result<(), Box<dyn Error>> {
        let value = bincode::serialize(value)?;
        self.tree.insert(key, value)?;
        Ok(())
    }

    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<T>, Box<dyn Error>> {
        match self.tree.get(key)? {
            Some(bytes) => Ok(Some(bincode::deserialize::<T>(&bytes)?)),
            None => Ok(None),
        }
    }

    pub fn remove(&self, key: impl AsRef<[u8]>) -> Result<(), Box<dyn Error>> {
        self.tree.remove(key)?;
        Ok(())
    }
}
