use super::Database;
use std::{collections::HashMap, fmt::Display};

type Value = IVec;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Binding(String);

impl Binding {
    pub fn new(label: impl AsRef<str>) -> Self {
        Self(label.as_ref().to_owned())
    }
}

#[derive(Debug, Clone)]
pub struct BindingSet(HashMap<Binding, Value>);

impl BindingSet {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn constrain(mut self, binding: Binding, value: Value) -> Self {
        self.0.insert(binding, value);
        self
    }

    pub fn get(&self, binding: &Binding) -> Option<&Value> {
        self.0.get(binding)
    }

    pub fn take(&mut self, binding: &Binding) -> Option<Value> {
        self.0.remove(&binding)
    }
}

impl Display for BindingSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ ")?;

        for (i, (key, value)) in self.0.iter().enumerate() {
            write!(f, "{} = {value:?}", key.0)?;
            if i < self.0.len() - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, " }}")
    }
}

impl Database {
    pub fn query(&self, rules: Vec<Rule>) -> Vec<BindingSet> {
        let mut valid_sets = Vec::new();
        self.constrain_binding_sets(BindingSet::new(), rules, &mut valid_sets);
        valid_sets
    }

    fn constrain_binding_sets(
        &self,
        set: BindingSet,
        mut rules: Vec<Rule>,
        valid_sets: &mut Vec<BindingSet>,
    ) {
        match rules.pop() {
            Some(rule) => rule.constrain(set, self).into_iter().for_each(|new_set| {
                self.constrain_binding_sets(new_set, rules.clone(), valid_sets);
            }),
            None => valid_sets.push(set),
        }
    }
}

#[derive(Clone)]
pub enum RuleVar {
    Binding(Binding),
    Constant(Value),
}

enum RuleVariant {
    Constant,
    Single(Value, Value, Binding),
    Double(Value, Binding, Binding),
    Triple(Binding, Binding, Binding),
}

#[derive(Clone)]
pub struct Rule {
    entity: RuleVar,
    attribute: RuleVar,
    value: RuleVar,
}

impl Rule {
    pub fn new(entity: RuleVar, attribute: RuleVar, value: RuleVar) -> Self {
        Self {
            entity,
            attribute,
            value,
        }
    }

    fn constrain(mut self, set: BindingSet, db: &Database) -> Vec<BindingSet> {
        use RuleVar::*;
        use RuleVariant::{Double, Single, Triple};

        // Replace any bound bindings in the rule
        Rule::load_binding(&mut self.entity, &set);
        Rule::load_binding(&mut self.attribute, &set);
        Rule::load_binding(&mut self.value, &set);

        // Map the binding combination to a tree and its corresponding rule variant
        let (tree, variant) = match (self.entity, self.attribute, self.value) {
            // Constants only
            (Constant(_), Constant(_), Constant(_)) => (&db.eav, RuleVariant::Constant),

            // 1 binding
            (Binding(entity), Constant(attribute), Constant(value)) => {
                (&db.vae, Single(value, attribute, entity))
            }
            (Constant(entity), Constant(attribute), Binding(value)) => {
                (&db.eav, Single(entity, attribute, value))
            }
            (Constant(_), Binding(_), Constant(_)) => unimplemented!(),

            // 2 bindings
            (Binding(entity), Binding(attribute), Constant(value)) => {
                (&db.vae, Double(value, attribute, entity))
            }
            (Constant(entity), Binding(attribute), Binding(value)) => {
                (&db.eav, Double(entity, attribute, value))
            }
            (Binding(entity), Constant(attribute), Binding(value)) => {
                (&db.ave, Double(attribute, value, entity))
            }

            // 3 bindings
            (Binding(entity), Binding(attribute), Binding(value)) => {
                (&db.eav, Triple(entity, attribute, value))
            }
        };

        // Build sets based on the variant
        // TODO Get rid of unwraps ...
        match variant {
            RuleVariant::Constant => vec![set],

            Single(x, y, z) => tree
                .scan_key2_prefix(&x, &y)
                .map(|r| r.unwrap())
                .filter(|(_, val_y, _)| *val_y == y)
                .flat_map(move |(_, _, vals_z)| {
                    let set = set.clone();
                    let z = z.clone();
                    vals_z.map(move |val_z| set.clone().constrain(z.clone(), val_z))
                })
                .collect(),

            Double(x, y, z) => tree
                .scan_key2_prefix(&x, [])
                .map(|r| r.unwrap())
                .flat_map(move |(_, val_y, vals_z)| {
                    let set = set.clone();
                    let y = y.clone();
                    let z = z.clone();
                    vals_z.map(move |val_z| {
                        set.clone()
                            .constrain(y.clone(), val_y.clone())
                            .constrain(z.clone(), val_z)
                    })
                })
                .collect(),

            Triple(x, y, z) => tree
                .scan()
                .map(|r| r.unwrap())
                .flat_map(move |(val_x, val_y, vals_z)| {
                    let set = set.clone();
                    let x = x.clone();
                    let y = y.clone();
                    let z = z.clone();

                    vals_z.map(move |val_z| {
                        set.clone()
                            .constrain(x.clone(), val_x.clone())
                            .constrain(y.clone(), val_y.clone())
                            .constrain(z.clone(), val_z)
                    })
                })
                .collect(),
        }
    }

    fn load_binding(var: &mut RuleVar, set: &BindingSet) {
        if let RuleVar::Binding(binding) = var {
            if let Some(value) = set.get(binding) {
                *var = RuleVar::Constant(value.clone());
            }
        }
    }
}

impl From<&Binding> for RuleVar {
    fn from(binding: &Binding) -> Self {
        Self::Binding(binding.clone())
    }
}

impl From<Binding> for RuleVar {
    fn from(binding: Binding) -> Self {
        Self::Binding(binding)
    }
}

impl<T> From<T> for RuleVar
where
    T: AsRef<[u8]>,
{
    fn from(value: T) -> Self {
        Self::Constant(value.as_ref().into())
    }
}

#[macro_export]
macro_rules! bindings {
    ($(?$binding:ident),*) => {
        $(let $binding = Binding::new(stringify!($binding));)*
    };
}

#[macro_export]
macro_rules! query {
    ($db:expr, [$([$entity:expr, $attr:expr, $value:expr]),*]) => {
        {
            let rules = vec![
                $(Rule::new((&$entity).into(), (&$attr).into(), (&$value).into()),)*
            ];

            $db.query(rules)
        }
    };
}
