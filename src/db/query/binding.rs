use super::super::{Attribute, Entity, Value};
use std::{collections::HashMap, fmt, marker::PhantomData, sync::atomic::AtomicU64};

static COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Variable<T>(String, PhantomData<T>);

#[derive(Default, Clone, Debug)]
pub struct VariableSet {
    entity: HashMap<Variable<Entity>, Entity>,
    attribute: HashMap<Variable<Attribute>, Attribute>,
    value: HashMap<Variable<Value>, Value>,
}

impl<T> Variable<T> {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into(), PhantomData)
    }
}

impl From<Variable<Value>> for Variable<Entity> {
    fn from(value: Variable<Value>) -> Self {
        Variable::new(&value.0)
    }
}

impl From<Variable<Entity>> for Variable<Value> {
    fn from(value: Variable<Entity>) -> Self {
        Variable::new(&value.0)
    }
}

pub trait VariableSetExt<T> {
    fn constrain(&self, variable: &Variable<T>, entry: T) -> Self;
    fn get(&self, variable: &Variable<T>) -> Option<&T>;
}

impl VariableSetExt<Entity> for VariableSet {
    fn constrain(&self, variable: &Variable<Entity>, entry: Entity) -> Self {
        let mut instance = self.clone();

        // Prevent two variables from having the same binding
        if instance.entity.values().find(|v| **v == entry).is_none() {
            instance.entity.insert(variable.clone(), entry.clone());
        }

        let value = Value::Reference(entry);
        if instance.value.values().find(|v| **v == value).is_none() {
            instance.value.insert(variable.clone().into(), value);
        }

        instance
    }

    fn get(&self, variable: &Variable<Entity>) -> Option<&Entity> {
        self.entity.get(variable)
    }
}

impl VariableSetExt<Attribute> for VariableSet {
    fn constrain(&self, variable: &Variable<Attribute>, entry: Attribute) -> Self {
        let mut instance = self.clone();

        // Prevent two variables from having the same binding
        if instance.attribute.values().find(|v| **v == entry).is_none() {
            instance.attribute.insert(variable.clone(), entry);
        }

        instance
    }

    fn get(&self, variable: &Variable<Attribute>) -> Option<&Attribute> {
        self.attribute.get(variable)
    }
}

impl VariableSetExt<Value> for VariableSet {
    fn constrain(&self, variable: &Variable<Value>, entry: Value) -> Self {
        let mut instance = self.clone();

        if let Value::Reference(entity) = &entry {
            if instance.entity.values().find(|v| *v == entity).is_none() {
                instance
                    .entity
                    .insert(variable.clone().into(), entity.clone());
            }
        }

        // Prevent two variables from having the same binding
        if instance.value.values().find(|v| **v == entry).is_none() {
            instance.value.insert(variable.clone(), entry);
        }

        instance
    }

    fn get(&self, variable: &Variable<Value>) -> Option<&Value> {
        self.value.get(variable)
    }
}

impl fmt::Debug for Variable<Entity> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl fmt::Debug for Variable<Attribute> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.0)
    }
}

impl fmt::Debug for Variable<Value> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "?{}", self.0)
    }
}
