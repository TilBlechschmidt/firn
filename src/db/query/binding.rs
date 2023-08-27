use super::super::{Attribute, Entity, Value};
use std::{
    collections::HashMap,
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Variable {
    Named(String),
    Unnamed(u64),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum Binding {
    /// Can be used as literally and inside Value::Reference
    Entity(Entity),
    Attribute(Attribute),
    /// Variant of value
    Data(String),
}

#[derive(Default, Clone, Debug)]
pub struct BindingSet<'v>(HashMap<&'v Variable, Binding>);

impl Variable {
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    pub fn unnamed() -> Self {
        Self::Unnamed(COUNTER.fetch_add(1, Ordering::Acquire))
    }
}

impl<'v> BindingSet<'v> {
    pub fn constrained(&self, variable: &'v Variable, binding: impl Into<Binding>) -> Self {
        let mut copy = self.clone();
        copy.0.insert(variable, binding.into());
        copy
    }

    pub fn purge_unnamed(&mut self) {
        let unnamed = self
            .0
            .keys()
            .filter(|v| {
                if let Variable::Unnamed(_) = v {
                    true
                } else {
                    false
                }
            })
            .map(|v| *v)
            .collect::<Vec<_>>();

        for var in unnamed {
            self.0.remove(var);
        }
    }

    pub fn get(&self, variable: &Variable) -> Option<&Binding> {
        self.0.get(variable)
    }
}

impl From<Entity> for Binding {
    fn from(value: Entity) -> Self {
        Self::Entity(value)
    }
}

impl From<Attribute> for Binding {
    fn from(value: Attribute) -> Self {
        Self::Attribute(value)
    }
}

impl From<Value> for Binding {
    fn from(value: Value) -> Self {
        match value {
            Value::Reference(entity) => Self::from(entity),
            Value::Data(data) => Self::Data(data),
        }
    }
}

impl fmt::Debug for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Variable::Named(name) => write!(f, "?{name}"),
            Variable::Unnamed(id) => write!(f, "?{id}"),
        }
    }
}

impl fmt::Debug for Binding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Binding::Entity(entity) => write!(f, "{entity:?}"),
            Binding::Attribute(attribute) => write!(f, "{attribute:?}"),
            Binding::Data(data) => write!(f, "\"{data}\""),
        }
    }
}
