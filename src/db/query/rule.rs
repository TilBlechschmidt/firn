use super::{
    binding::{Variable, VariableSetExt},
    VariableSet,
};
use crate::db::{triplet_tree::TripletTree, Attribute, Database, Entity, Value};
use std::{borrow::Cow, hash::Hash};

enum ResolveVariant<'v, A, B, C> {
    Single(A, B, &'v Variable<C>),
    Double(A, &'v Variable<B>, &'v Variable<C>),
    Triple(&'v Variable<A>, &'v Variable<B>, &'v Variable<C>),
}

// This is mostly a convenience helper to make the rule definition a bit nicer
// by allowing the straight use of constants without inserting them into the VariableSet
#[derive(Clone)]
pub enum RuleVal<'v, T: Clone> {
    Variable(Cow<'v, Variable<T>>),
    Constant(T),
}

impl<'v, T: Clone> RuleVal<'v, T>
where
    VariableSet: VariableSetExt<T>,
{
    fn load(&self, set: &VariableSet) -> Self {
        if let RuleVal::Variable(var) = &self {
            if let Some(value) = set.get(&var) {
                return RuleVal::Constant(value.clone());
            }
        }

        self.clone()
    }
}

impl<'v, T: Clone> From<&T> for RuleVal<'v, T> {
    fn from(value: &T) -> Self {
        Self::Constant(value.clone())
    }
}

impl<'v, T> From<T> for RuleVal<'v, Entity>
where
    Entity: From<T>,
{
    fn from(value: T) -> Self {
        Self::Constant(Entity::from(value))
    }
}

impl<'v, T> From<T> for RuleVal<'v, Attribute>
where
    Attribute: From<T>,
{
    fn from(value: T) -> Self {
        Self::Constant(Attribute::from(value))
    }
}

impl<'v, T> From<T> for RuleVal<'v, Value>
where
    Value: From<T>,
{
    fn from(value: T) -> Self {
        Self::Constant(Value::from(value))
    }
}

impl<'v> From<&'v Variable<Entity>> for RuleVal<'v, Entity> {
    fn from(value: &'v Variable<Entity>) -> Self {
        Self::Variable(Cow::Borrowed(value))
    }
}

impl<'v> From<&'v Variable<Attribute>> for RuleVal<'v, Attribute> {
    fn from(value: &'v Variable<Attribute>) -> Self {
        Self::Variable(Cow::Borrowed(value))
    }
}

impl<'v> From<&'v Variable<Value>> for RuleVal<'v, Value> {
    fn from(value: &'v Variable<Value>) -> Self {
        Self::Variable(Cow::Borrowed(value))
    }
}

impl<'v> From<&'v Variable<Entity>> for RuleVal<'v, Value> {
    fn from(value: &'v Variable<Entity>) -> Self {
        Self::Variable(Cow::Owned(value.clone().into()))
    }
}

pub struct Rule<'v> {
    entity: RuleVal<'v, Entity>,
    attribute: RuleVal<'v, Attribute>,
    value: RuleVal<'v, Value>,
}

impl<'v> Rule<'v> {
    pub fn new(
        entity: impl Into<RuleVal<'v, Entity>>,
        attribute: impl Into<RuleVal<'v, Attribute>>,
        value: impl Into<RuleVal<'v, Value>>,
    ) -> Self {
        Self {
            entity: entity.into(),
            attribute: attribute.into(),
            value: value.into(),
        }
    }

    pub(super) fn constrain(&self, set: VariableSet, db: &Database) -> Vec<VariableSet> {
        use ResolveVariant::{Double, Single, Triple};
        use RuleVal::*;

        // Map the binding combination to a tree and its corresponding resolve variant
        match (
            self.entity.load(&set),
            self.attribute.load(&set),
            self.value.load(&set),
        ) {
            // Constants only which are already bound so nothing to do here
            (Constant(_), Constant(_), Constant(_)) => vec![set],

            // 1 variable
            (Variable(entity), Constant(attribute), Constant(value)) => {
                db.vae.constrain(set, Single(value, attribute, &entity))
            }
            (Constant(entity), Constant(attribute), Variable(value)) => {
                db.eav.constrain(set, Single(entity, attribute, &value))
            }
            (Constant(_), Variable(_), Constant(_)) => unimplemented!(),

            // 2 variables
            (Variable(entity), Variable(attribute), Constant(value)) => {
                db.vae.constrain(set, Double(value, &attribute, &entity))
            }
            (Constant(entity), Variable(attribute), Variable(value)) => {
                db.eav.constrain(set, Double(entity, &attribute, &value))
            }
            (Variable(entity), Constant(attribute), Variable(value)) => {
                db.ave.constrain(set, Double(attribute, &value, &entity))
            }

            // 3 variables
            (Variable(entity), Variable(attribute), Variable(value)) => {
                db.eav.constrain(set, Triple(&entity, &attribute, &value))
            }
        }
    }
}

trait Constrain<A, B, C> {
    fn constrain(&self, set: VariableSet, variant: ResolveVariant<A, B, C>) -> Vec<VariableSet>;
}

impl<A: Hash + Eq + Clone, B: Hash + Eq + Clone, C: Clone> Constrain<A, B, C>
    for TripletTree<A, B, C>
where
    VariableSet: VariableSetExt<A>,
    VariableSet: VariableSetExt<B>,
    VariableSet: VariableSetExt<C>,
{
    fn constrain(&self, set: VariableSet, variant: ResolveVariant<A, B, C>) -> Vec<VariableSet> {
        use ResolveVariant::*;

        match variant {
            Single(a, b, c) => self
                .values(&a, &b)
                .cloned()
                .map(|binding| set.constrain(&c, binding))
                .collect(),

            Double(a, b, c) => self
                .get(&a)
                .flat_map(|(v_b, v)| {
                    v.iter()
                        .map(|v_c| set.constrain(&b, v_b.clone()).constrain(&c, v_c.clone()))
                })
                .collect(),

            Triple(a, b, c) => self
                .scan()
                .flat_map(|(v_a, v_b, v)| {
                    v.iter().map(|v_c| {
                        set.constrain(&a, v_a.clone())
                            .constrain(&b, v_b.clone())
                            .constrain(&c, v_c.clone())
                    })
                })
                .collect(),
        }
    }
}
