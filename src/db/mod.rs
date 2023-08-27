#![allow(dead_code)]

use std::{fmt, sync::mpsc};
use triplet_tree::TripletTree;

mod query;
mod triplet_tree;

// pub use query::*;
// use crate::{bindings, query};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Entity(pub String);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Attribute {
    pub namespace: String,
    pub entry: String,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Value {
    Reference(Entity),
    Data(String),
}

impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}/{}", self.namespace, self.entry)
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl From<String> for Entity {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Entity {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::Data(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::Data(value.to_string())
    }
}

impl From<Entity> for Value {
    fn from(entity: Entity) -> Self {
        Self::Reference(entity)
    }
}

impl From<&Entity> for Value {
    fn from(entity: &Entity) -> Self {
        Self::Reference(entity.clone())
    }
}

#[macro_export]
macro_rules! attribute {
    ($namespace:ident / $entry:ident) => {
        Attribute {
            namespace: String::from(stringify!($namespace)),
            entry: String::from(stringify!($entry)),
        }
    };
}

pub struct Database {
    pub eav: TripletTree<Entity, Attribute, Value>,
    // aev: TripletTree,
    pub ave: TripletTree<Attribute, Value, Entity>,
    pub vae: TripletTree<Value, Attribute, Entity>,

    write_log: mpsc::Sender<(Entity, Attribute, Value)>,
}

impl Database {
    pub fn new() -> (Self, mpsc::Receiver<(Entity, Attribute, Value)>) {
        let eav = TripletTree::default();
        let ave = TripletTree::default();
        let vae = TripletTree::default();
        let (write_log, rx) = mpsc::channel();

        (
            Self {
                eav,
                ave,
                vae,
                write_log,
            },
            rx,
        )
    }

    pub fn insert(
        &mut self,
        entity: impl Into<Entity>,
        attribute: impl Into<Attribute>,
        value: impl Into<Value>,
    ) {
        let entity = entity.into();
        let attribute = attribute.into();
        let value = value.into();

        self.write_log
            .send((entity.clone(), attribute.clone(), value.clone()))
            .ok();

        self.eav
            .append(entity.clone(), attribute.clone(), value.clone());
        self.insert_into_indices(entity, attribute, value);
    }

    fn insert_into_indices(&mut self, entity: Entity, attribute: Attribute, value: Value) {
        // self.aev.append(&attribute, &entity, &value)?;
        self.ave
            .append(attribute.clone(), value.clone(), entity.clone());
        self.vae.append(value, attribute, entity);
    }

    pub fn rebuild_indices(&mut self) {
        // self.aev.clear()?;
        self.ave.clear();
        self.vae.clear();

        for entry in self.eav.scan() {
            let (entity, attribute, values) = entry;

            for value in values {
                // Code duplication here bc the borrow checker wouldn't be happy otherwise :(
                self.ave
                    .append(attribute.clone(), value.clone(), entity.clone());
                self.vae
                    .append(value.clone(), attribute.clone(), entity.clone());
            }
        }
    }

    pub fn get(&self, entity: &Entity, attribute: &Attribute) -> impl Iterator<Item = &Value> {
        self.eav.values(entity, attribute)
        // bindings![?value];
        // query!(self, [[entity, attribute, value]])
        //     .into_iter()
        //     .flat_map(move |mut set| set.take(&value))
    }
}
