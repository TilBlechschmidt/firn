use sled::Tree;
use std::{
    io::Read,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct WriteLog {
    tree: Tree,
    counter: AtomicUsize,
}

impl WriteLog {
    /// Constructs a new TripletTree from a Tree. Cheap to use.
    /// TripletTree operations will panic if the merge operator has not been set.
    pub fn new(tree: Tree) -> Self {
        Self {
            tree,
            counter: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, entity: &str, attribute: &str, value: &str) -> sled::Result<()> {
        let key = self.counter.fetch_add(1, Ordering::SeqCst).to_be_bytes();

        let buffer = WriteLog::pack(entity, attribute, value);
        self.tree.insert(key, buffer)?;

        Ok(())
    }

    pub fn pop(&self) -> sled::Result<Option<(String, String, String)>> {
        match self.tree.iter().next() {
            None => Ok(None),
            Some(result) => {
                let (key, value) = result?;
                self.tree.remove(key)?;
                Ok(Some(WriteLog::unpack(&value)))
            }
        }
    }

    fn pack(entity: &str, attribute: &str, value: &str) -> Vec<u8> {
        let len_entity = entity.as_bytes().len();
        let len_attribute = attribute.as_bytes().len();

        let mut buf = Vec::with_capacity(len_entity + len_attribute);

        leb128::write::unsigned(&mut buf, len_entity as u64).expect("failed to write to vector");
        leb128::write::unsigned(&mut buf, len_attribute as u64).expect("failed to write to vector");

        buf.extend_from_slice(entity.as_bytes());
        buf.extend_from_slice(attribute.as_bytes());
        buf.extend_from_slice(value.as_bytes());

        return buf;
    }

    fn unpack(buffer: &[u8]) -> (String, String, String) {
        let mut buf = buffer.as_ref();

        let len_entity = leb128::read::unsigned(&mut buf)
            .expect("failed to read entity length while unpacking")
            as usize;

        let len_attribute = leb128::read::unsigned(&mut buf)
            .expect("failed to read attribute length while unpacking")
            as usize;

        let mut entity = vec![0; len_entity];
        let mut attribute = vec![0; len_attribute];
        let mut value = Vec::with_capacity(buf.len());

        buf.read_exact(&mut entity)
            .expect("failed to unpack entity");

        buf.read_exact(&mut attribute)
            .expect("failed to unpack attribute");

        buf.read_to_end(&mut value).expect("failed to unpack value");

        (
            String::from_utf8(entity).expect("entity is invalid UTF-8"),
            String::from_utf8(attribute).expect("attribute is invalid UTF-8"),
            String::from_utf8(value).expect("value is invalid UTF-8"),
        )
    }
}

#[cfg(test)]
mod does {
    use super::*;

    #[test]
    fn pack_correctly() {
        let entity = "Hello";
        let attribute = "world";
        let value = "!";

        let packed = WriteLog::pack(&entity, &attribute, &value);
        let (e, a, v) = WriteLog::unpack(&packed);

        assert_eq!(e, entity);
        assert_eq!(a, attribute);
        assert_eq!(v, value);
    }

    // TODO We should probably test whether the write log actually works :P
}
