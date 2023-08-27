use super::Database;

mod binding;
mod rule;

pub use binding::*;
pub use rule::*;

impl Database {
    pub fn query<'v>(
        &self,
        rules: &'v Vec<Rule<'v>>,
        bindings: BindingSet<'v>,
    ) -> Vec<BindingSet<'v>> {
        let mut valid_sets = Vec::new();
        self.constrain_binding_sets(bindings, &rules, &mut valid_sets);

        valid_sets
            .into_iter()
            .map(|mut set| {
                set.purge_unnamed();
                set
            })
            .collect()
    }

    fn constrain_binding_sets<'v>(
        &self,
        set: BindingSet<'v>,
        rules: &'v [Rule<'v>],
        valid_sets: &mut Vec<BindingSet<'v>>,
    ) {
        match rules.first() {
            Some(rule) => rule.constrain(set, self).into_iter().for_each(|new_set| {
                self.constrain_binding_sets(new_set, &rules[1..], valid_sets);
            }),
            None => valid_sets.push(set),
        }
    }
}

#[cfg(test)]
mod does {
    use super::super::*;
    use super::*;
    use crate::attribute;

    #[test]
    fn create_unique_variables() {
        let a = Variable::unnamed();
        let b = Variable::unnamed();
        assert_ne!(a, b);
    }

    #[test]
    fn work() {
        let (mut db, _) = Database::new();

        // Insert entity #1
        db.insert("1", attribute!(time / stamp), "42");
        db.insert("1", attribute!(doc / size), "255");

        // Insert entity #2
        db.insert("2", attribute!(time / stamp), "42");
        db.insert("2", attribute!(doc / size), "37");

        // Insert entity #3
        db.insert("3", attribute!(time / stamp), "1337");
        db.insert("3", attribute!(doc / size), "37");

        // Build a query
        let entity = Variable::named("entity");
        let size = Variable::named("size");

        let const_attr_timestamp = Variable::unnamed();
        let const_attr_size = Variable::unnamed();
        let const_val_timestamp = Variable::unnamed();

        let rules = vec![
            Rule::new(&entity, &const_attr_timestamp, &const_val_timestamp),
            Rule::new(&entity, &const_attr_size, &size),
        ];

        let bindings = BindingSet::default()
            .constrained(&const_attr_timestamp, attribute!(time / stamp))
            .constrained(&const_attr_size, attribute!(doc / size))
            .constrained(&const_val_timestamp, Value::Data("42".into()));

        let results = db.query(&rules, bindings);
        results.iter().for_each(|b| println!("{b:?}"));

        // Bit annoying due to the enum based variable API but oh well, deadline is calling
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].get(&entity), Some(&Binding::Entity("1".into())));
        assert_eq!(results[0].get(&size), Some(&Binding::Data("255".into())));
        assert_eq!(results[1].get(&entity), Some(&Binding::Entity("2".into())));
        assert_eq!(results[1].get(&size), Some(&Binding::Data("37".into())));
    }
}
