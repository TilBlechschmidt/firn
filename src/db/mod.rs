#![allow(dead_code)]

use std::{fmt, sync::mpsc};
use triplet_tree::TripletTree;

mod query;
mod triplet_tree;

pub use query::*;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Entity(pub String);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Attribute(pub String);

pub type Data = String;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Value {
    Reference(Entity),
    Data(Data),
}

impl Value {
    /// Attempts to borrow the contained data but panics if this value contains a reference.
    ///
    /// This is usually helpful when using the `query!` macro as it automatically turns
    /// variables containing references into entity variables and thus all value vars contain data.
    pub fn data(&self) -> &str {
        match self {
            Value::Reference(_) => {
                panic!("attempted to take data from value that contained a reference")
            }
            Value::Data(data) => data.as_str(),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Reference(entity) => write!(f, "Ref({})", entity.0),
            Value::Data(data) => write!(f, "{data}"),
        }
    }
}

impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<T> for Entity
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl<T> From<T> for Attribute
where
    T: ToString,
{
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}

impl<T> From<T> for Value
where
    T: ToString,
{
    fn from(value: T) -> Self {
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

    pub fn len(&self) -> usize {
        self.eav.len()
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
