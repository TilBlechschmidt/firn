#![allow(dead_code)]

use sled::{Db, IVec, Mode};
use triplet_tree::TripletTree;
use write_log::WriteLog;

mod query;
mod triplet_tree;
mod write_log;

pub use query::*;

use crate::{bindings, query};

// Triplets are stored as ((entity_length, entity, attribute), [value]) tuples.
//
// fn main() -> Result<(), sled::Error> {
//     // Create instance
//     let mut db = Database::temporary()?;
//
//     // Insert entity #1
//     db.insert("1", ":time/stamp", "42")?;
//     db.insert("1", ":doc/size", [255])?;
//
//     // Insert entity #2
//     db.insert("2", ":time/stamp", "42")?;
//     db.insert("2", ":doc/size", [37])?;
//
//     // Build a query
//     let rules = find!(?entity, ?size :where [
//         [entity, ":time/stamp", "42"],
//         [entity, ":doc/size", size]
//     ]);
//
//     // Execute and print
//     db.query(rules).iter().for_each(println);
//
//     // Output:
//     //  { size = [37], entity = "2" }
//     //  { size = [255], entity = "1" }
//
//     Ok(())
// }
//
// fn println<T: Display>(value: T) {
//     println!("{value}");
// }
pub struct Database {
    pub eav: TripletTree,
    // aev: TripletTree,
    pub ave: TripletTree,
    pub vae: TripletTree,

    write_log: WriteLog,
}

impl Database {
    pub fn temporary() -> sled::Result<Self> {
        Self::new(
            sled::Config::new()
                .temporary(true)
                .mode(Mode::HighThroughput)
                .open()?,
        )
    }

    pub fn new(db: Db) -> sled::Result<Self> {
        let eav = TripletTree::new(db.open_tree("eav")?);
        let ave = TripletTree::new(db.open_tree("ave")?);
        let vae = TripletTree::new(db.open_tree("vae")?);
        let write_log = WriteLog::new(db.open_tree("write_log")?);

        Ok(Self {
            eav,
            ave,
            vae,
            write_log,
        })
    }

    pub fn insert(
        &self,
        entity: impl AsRef<str>,
        attribute: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> sled::Result<()> {
        self.write_log
            .push(entity.as_ref(), attribute.as_ref(), value.as_ref())?;

        let entity = entity.as_ref().as_bytes();
        let attribute = attribute.as_ref().as_bytes();
        let value = value.as_ref().as_bytes();

        self.eav.append(&entity, &attribute, &value)?;
        self.insert_into_indices(entity, attribute, value)
    }

    fn insert_into_indices(
        &self,
        entity: impl AsRef<[u8]>,
        attribute: impl AsRef<[u8]>,
        value: impl AsRef<[u8]>,
    ) -> sled::Result<()> {
        // self.aev.append(&attribute, &entity, &value)?;
        self.ave.append(&attribute, &value, &entity)?;
        self.vae.append(&value, &attribute, &entity)?;
        Ok(())
    }

    pub fn rebuild_indices(&self) -> sled::Result<()> {
        // self.aev.clear()?;
        self.ave.clear()?;
        self.vae.clear()?;

        for entry in self.eav.scan_key2_prefix([], []) {
            let (entity, attribute, values) = entry?;

            for value in values {
                self.insert_into_indices(&entity, &attribute, value)?;
            }
        }

        Ok(())
    }

    pub fn get(
        &self,
        entity: impl AsRef<[u8]>,
        attribute: impl AsRef<[u8]>,
    ) -> impl Iterator<Item = IVec> {
        bindings![?value];
        query!(self, [[entity, attribute, value]])
            .into_iter()
            .flat_map(move |mut set| set.take(&value))
    }

    // This function is mutable to prevent regular users of an instance from taking stuff, cheesy I know but oh well
    pub fn pop_from_log(&mut self) -> sled::Result<Option<(String, String, String)>> {
        self.write_log.pop()
    }
}

fn length_prefixed_concatenate_merge(
    _key: &[u8],
    old_value: Option<&[u8]>,
    merged_bytes: &[u8],
) -> Option<Vec<u8>> {
    let mut ret = old_value.map(<[_]>::to_vec).unwrap_or_default();

    leb128::write::unsigned(&mut ret, merged_bytes.len() as u64)
        .expect("failed to write to vector");

    ret.extend_from_slice(merged_bytes);

    Some(ret)
}
