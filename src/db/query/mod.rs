use super::Database;

mod binding;
mod rule;

pub use binding::*;
pub use rule::*;

impl Database {
    #[must_use]
    pub fn query<'v>(&self, rules: &'v Vec<Rule<'v>>) -> Vec<VariableSet> {
        let mut valid_sets = Vec::new();
        self.constrain_binding_sets(VariableSet::default(), &rules, &mut valid_sets);
        valid_sets
    }

    fn constrain_binding_sets<'v>(
        &self,
        set: VariableSet,
        rules: &'v [Rule<'v>],
        valid_sets: &mut Vec<VariableSet>,
    ) {
        match rules.first() {
            Some(rule) => rule.constrain(set, self).into_iter().for_each(|new_set| {
                self.constrain_binding_sets(new_set, &rules[1..], valid_sets);
            }),
            None => valid_sets.push(set),
        }
    }
}

#[macro_export]
macro_rules! query {
    (@var ?$name:ident) => {
        let $name = Variable::<Value>::new(stringify!($name));
    };

    (@var :$name:ident) => {
        let $name = Variable::<Attribute>::new(stringify!($name));
    };

    (@var #$name:ident) => {
        let $name = Variable::<Entity>::new(stringify!($name));
    };

    (@vars $key:tt$name:ident) => {
        query!(@var $key$name)
    };

    (@vars $key:tt$name:ident, $($leftover:tt)*) => {
        query!(@var $key$name);
        query!(@vars $($leftover)*);
    };

    // TODO Find a way to not borrow literals, preferably without writing handlers for all combinations
    (@rule #$entity:expr, :$attr:expr, ?$value:expr) => {
        Rule::new(&$entity, &$attr, &$value)
    };

    (@rule #$entity:expr, :$attr:expr, #$value:expr) => {
        Rule::new(&$entity, &$attr, &$value)
    };

    (@rules $({ $($rule:tt)+ }),*) => {
        vec![
            $(query!(@rule $($rule)*),)*
        ]
    };

    ($db:ident where ($($variables:tt)*) match [$($rules:tt)*] => $results:ident) => {
        query!(@vars $($variables)*);

        let $results = {
            let rules = query!(@rules $($rules)*);
            $db.query(&rules)
        };
    }
}

#[macro_export]
macro_rules! print_variables {
    ($result:expr => $($var:expr),*) => {
        {
            use $crate::db::VariableSetExt;

            let debug_string = vec![
                $(
                    format!(
                        "{:?}: {}",
                        &$var,
                        $result.get(&$var).map(|v| format!("{v:?}")).unwrap_or_default()
                    )
                ),*
            ].join(", ");

            println!("{{ {debug_string} }}");

            debug_string
        }
    };
}

#[macro_export]
macro_rules! print_results {
    ($results:expr => $($var:expr),*) => {
        for result in $results.iter() {
            $crate::print_variables!(result => $($var),*);
        }
    };
}

#[cfg(test)]
mod does {
    use super::super::*;
    use super::*;

    #[test]
    fn work() {
        let (mut db, _) = Database::new();

        // Insert entity #1
        db.insert("1", "time/stamp", "42");
        db.insert("1", "doc/size", "255");

        // Insert entity #2
        db.insert("2", "time/stamp", "42");
        db.insert("2", "doc/size", "37");

        // Insert entity #3
        db.insert("3", "time/stamp", "1337");
        db.insert("3", "doc/size", "37");

        // Build a query
        let entity = Variable::new("entity");
        let size = Variable::<Value>::new("size");

        let rules = vec![
            Rule::new(&entity, "time/stamp", "42"),
            Rule::new(&entity, "doc/size", &size),
        ];

        let results = db.query(&rules);
        results.iter().for_each(|b| println!("{b:?}"));

        // Bit annoying due to the enum based variable API but oh well, deadline is calling
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].get(&entity), Some(&"1".into()));
        assert_eq!(results[0].get(&size), Some(&"255".into()));
        assert_eq!(results[1].get(&entity), Some(&"2".into()));
        assert_eq!(results[1].get(&size), Some(&"37".into()));
    }

    #[test]
    fn handle_coercion() {
        let (mut db, _) = Database::new();

        // Insert entity #1
        db.insert("1", "time/stamp", "42");
        db.insert("1", "doc/size", "255");

        // Insert entity #2
        db.insert("2", "time/stamp", "42");
        db.insert("2", "doc/size", "37");

        // Insert entity #3
        db.insert("3", "rel/something", Value::Reference("1".into()));
        db.insert("3", "doc/size", "69");

        // Build a query
        let entity_a = Variable::new("src");
        let entity_b = Variable::new("dst");
        let size = Variable::<Value>::new("size");

        let rules = vec![
            Rule::new(&entity_a, "time/stamp", "42"),
            Rule::new(&entity_b, "rel/something", &entity_a),
            Rule::new(&entity_b, "doc/size", &size),
        ];

        let results = db.query(&rules);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get(&entity_a), Some(&"1".into()));
        assert_eq!(results[0].get(&entity_b), Some(&"3".into()));
        assert_eq!(results[0].get(&size), Some(&"69".into()));
    }

    #[test]
    fn codegen_correctly() {
        let (mut db, _) = Database::new();

        db.insert("1", "time/stamp", "42");
        db.insert("1", "doc/size", "255");

        query!(db where (?a, #b, ?c) match [
            { #b, :"time/stamp", ?a },
            { #b, :"doc/size", ?c }
        ] => results);

        print_results!(results => a, b, c);

        let results_str = print_variables!(results[0] => a, b, c);
        assert_eq!(results_str, "?a: 42, #b: 1, ?c: 255");
    }
}
