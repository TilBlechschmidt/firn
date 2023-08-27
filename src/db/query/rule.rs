use super::binding::{Binding, BindingSet, Variable};
use crate::db::{triplet_tree::TripletTree, Attribute, Database, Entity, Value};

#[derive(Debug)]
enum ResolveVariant<'v> {
    Single(Binding, Binding, &'v Variable),
    Double(Binding, &'v Variable, &'v Variable),
    Triple(&'v Variable, &'v Variable, &'v Variable),
}

#[derive(Debug)]
pub struct Rule<'v> {
    entity: &'v Variable,
    attribute: &'v Variable,
    value: &'v Variable,
}

impl<'v> Rule<'v> {
    pub fn new(entity: &'v Variable, attribute: &'v Variable, value: &'v Variable) -> Self {
        Self {
            entity,
            attribute,
            value,
        }
    }

    pub(super) fn constrain(&'v self, set: BindingSet<'v>, db: &Database) -> Vec<BindingSet<'v>> {
        use ResolveVariant::{Double, Single, Triple};

        let entity = self.entity;
        let attribute = self.attribute;
        let value = self.value;

        // Map the binding combination to a tree and its corresponding rule variant
        let (tree, variant): (&dyn Constrain, ResolveVariant) = match (
            set.get(self.entity).cloned(),
            set.get(self.attribute).cloned(),
            set.get(self.value).cloned(),
        ) {
            // Constants only which are already bound so nothing to do here
            (Some(_), Some(_), Some(_)) => return vec![set],

            // 1 variable
            (None, Some(attribute), Some(value)) => (&db.vae, Single(value, attribute, entity)),
            (Some(entity), Some(attribute), None) => (&db.eav, Single(entity, attribute, value)),
            (Some(_), None, Some(_)) => unimplemented!(),

            // 2 variables
            (None, None, Some(value)) => (&db.vae, Double(value, attribute, entity)),
            (Some(entity), None, None) => (&db.eav, Double(entity, attribute, value)),
            (None, Some(attribute), None) => (&db.ave, Double(attribute, value, entity)),

            // 3 variables
            (None, None, None) => (&db.eav, Triple(entity, attribute, value)),
        };

        tree.constrain(variant, set)
    }
}

trait Constrain {
    fn constrain<'v>(
        &self,
        variant: ResolveVariant<'v>,
        set: BindingSet<'v>,
    ) -> Vec<BindingSet<'v>>;
}

impl Constrain for TripletTree<Entity, Attribute, Value> {
    fn constrain<'v>(
        &self,
        variant: ResolveVariant<'v>,
        set: BindingSet<'v>,
    ) -> Vec<BindingSet<'v>> {
        use ResolveVariant::*;

        match variant {
            // Fetch all possible values and create a set for each of them
            Single(Binding::Entity(entity), Binding::Attribute(attribute), value) => self
                .values(&entity, &attribute)
                .cloned()
                .map(|binding| set.constrained(value, binding))
                .collect(),

            Double(Binding::Entity(entity), attribute, value) => self
                .get(&entity)
                .flat_map(|(a, values)| {
                    values.iter().map(|v| {
                        set.constrained(attribute, a.clone())
                            .constrained(value, v.clone())
                    })
                })
                .collect(),

            // Scan the full table and create a set for each e+a+[v] combo
            Triple(entity, attribute, value) => self
                .scan()
                .flat_map(|(e, a, values)| {
                    values.iter().map(|v| {
                        set.constrained(entity, e.clone())
                            .constrained(attribute, a.clone())
                            .constrained(value, v.clone())
                    })
                })
                .collect(),

            // Some binding does not have the correct type so truncate the tree here
            _ => vec![],
        }
    }
}

impl Constrain for TripletTree<Value, Attribute, Entity> {
    fn constrain<'v>(
        &self,
        variant: ResolveVariant<'v>,
        set: BindingSet<'v>,
    ) -> Vec<BindingSet<'v>> {
        use ResolveVariant::*;

        match variant {
            // -- Single variant for both ref & data
            Single(Binding::Entity(reference), Binding::Attribute(attribute), entity) => self
                .values(&reference.into(), &attribute)
                .cloned()
                .map(|binding| set.constrained(entity, binding))
                .collect(),

            Single(Binding::Data(data), Binding::Attribute(attribute), entity) => self
                .values(&data.as_str().into(), &attribute)
                .cloned()
                .map(|binding| set.constrained(entity, binding))
                .collect(),

            // -- Double variant for both ref & data
            Double(Binding::Entity(reference), attribute, entity) => self
                .get(&reference.into())
                .flat_map(|(a, entities)| {
                    entities.iter().map(|e| {
                        set.constrained(attribute, a.clone())
                            .constrained(entity, e.clone())
                    })
                })
                .collect(),

            Double(Binding::Data(data), attribute, entity) => self
                .get(&data.as_str().into())
                .flat_map(|(a, entities)| {
                    entities.iter().map(|e| {
                        set.constrained(attribute, a.clone())
                            .constrained(entity, e.clone())
                    })
                })
                .collect(),

            // -- Triple variant
            Triple(value, attribute, entity) => self
                .scan()
                .flat_map(|(v, a, entities)| {
                    entities.iter().map(|e| {
                        set.constrained(entity, e.clone())
                            .constrained(attribute, a.clone())
                            .constrained(value, v.clone())
                    })
                })
                .collect(),

            // Some binding does not have the correct type so truncate the tree here
            _ => vec![],
        }
    }
}

impl Constrain for TripletTree<Attribute, Value, Entity> {
    fn constrain<'v>(
        &self,
        variant: ResolveVariant<'v>,
        set: BindingSet<'v>,
    ) -> Vec<BindingSet<'v>> {
        use ResolveVariant::*;

        match variant {
            // -- Single variant for both ref & data
            Single(Binding::Attribute(attribute), Binding::Entity(reference), entity) => self
                .values(&attribute, &reference.into())
                .cloned()
                .map(|binding| set.constrained(entity, binding))
                .collect(),

            Single(Binding::Attribute(attribute), Binding::Data(data), entity) => self
                .values(&attribute, &data.as_str().into())
                .cloned()
                .map(|binding| set.constrained(entity, binding))
                .collect(),

            // -- Double variant for both ref & data
            Double(Binding::Attribute(attribute), value, entity) => self
                .get(&attribute)
                .flat_map(|(v, entities)| {
                    entities.iter().map(|e| {
                        set.constrained(value, v.clone())
                            .constrained(entity, e.clone())
                    })
                })
                .collect(),

            // Scan the full table and create a set for each e+a+[v] combo
            Triple(attribute, value, entity) => self
                .scan()
                .flat_map(|(a, v, entities)| {
                    entities.iter().map(|e| {
                        set.constrained(attribute, a.clone())
                            .constrained(value, v.clone())
                            .constrained(entity, e.clone())
                    })
                })
                .collect(),

            // Some binding does not have the correct type so truncate the tree here
            _ => vec![],
        }
    }
}
