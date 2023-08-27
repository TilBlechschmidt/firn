# Example 1 – One binding

```
[find ?e :where
    [?e, :doc/created_at, "2022-11-06"],
    [?e, :doc/cid, "8c90a9018bf2d8e13"]
]
```

1. Execute first rule, find all possible values for `e`
    - Uses ave or vae index
2. Execute second rule for each possible value of `e`
    - Uses ave or vae index
    - See which ones exist, purge those that do not
    - In this case it is a plain fetch

# Example 2 – Two bindings

```
[find ?p :where
    [?e, :doc/created_at, "2022-11-06"],
    [?e, :rel/predecessor, ?p]
]
```

1. Execute first rule, find all possible values for `e`
    - Uses ave or vae index
2. Execute second rule for each value of `e`
    - Find possible values for `p` given `e` using eav index
    - This forks into a 2D search
        - Outer loop iterates through values of `e`
        - Inner loop iterates through values of `p`
3. Return set of potential bindings for `e` & `p`
    - Filtered by what was asked to be returned
    - Filtering could be done earlier to remove/detect
        "orphaned" bindings that have no path to the output

# Example 3 – Three bindings

```
[find ?d :where
    [?e, :doc/created_at, "2022-11-06"],
    [?e, :rel/predecessor, ?p],
    [?p, :doc/created_at, ?d]
]
```

1. Same.
2. Same.
3. Run the next rule for each possible set of bindings for `p`
    - Add possible bindings for `d` to each set
    - Gradually populates binding combinations

# Example 4 – Constrained bindings

```
[find ?d :where
    [?e, :doc/created_at, "2022-11-06"],
    [?e, :rel/predecessor, ?p],
    [?p, :rank/stars, 4],
    [?p, :doc/created_at, ?d]
]
```

1. Populate binding values for `e`
2. Populate binding values for `p` (individually, not bound to `e` yet)
3. Filter binding sets by looking at which values for `e` & `p` satisfy the third rule
4. Same as before

# Algorithm V1: Recursive

1. Set all bindings as "unconstrained" (i.e. a set encompassing all possible values)
2. Take the first rule from the list
3. Apply the rule using the given binding constraints
    - In this case, just find all combinations as they are unconstrained
    - Rule returns a list of possible binding combinations
4. Repeat the following (recursively) for each binding combination
---
5. Take the second rule from the list
6. Apply the rule, further constraining the set of bindings
    - Given the constraints resulting from the first rule
---
7. Once no rules are left, push the remaining binding combination to a list
8. Filter the list by the bindings requested by the callee
9. Return the bindings

```rust
type Value = Vec<u8>;

struct Binding {
    identifier: String,
    requested: bool,
}

enum Constraint {
    Unconstrained,
    Constrained(Value),
}

struct BindingSet(HashMap<Binding, Constraint>);

enum RuleVar {
    Binding(Binding),
    Constant(Value)
}

impl Database {
    fn constrain_binding_sets(&mut self, set: BindingSet, mut rules: Vec<Rule>, valid_sets: &mut Vec<BindingSet>) {
        match rules.pop() {
            Some(rule) => rule.constrain(set, self).for_each(|new_set| {
                constrain_binding_sets(new_set, rules, valid_sets);
            }),
            None => valid_sets.push(set)
        }
    }
}

struct Rule {
    entity: RuleVar,
    attribute: RuleVar,
    value: RuleVar,
}

impl Rule {
    fn constrain(set: BindingSet, db: &mut Database) -> impl Iterator<Item = BindingSet> {
        // Example: 1 binding, works the same way regardless of position. Just change the index we are using!
        //
        //      [?e, :doc/created_at, "2022-11-06"] => vae
        //      [42, :doc/created_at, ?v]           => eav
        //      [42, ?a, "2022-11-06"]              => (eva?) probably an unlikely use-case thus no dedicated index
        //
        return db.vae.scan_key2_prefix("2022-11-06", ":doc/created_at")
            .filter(|(_, attribute, _)| attribute == ":doc/created_at")
            .map(|(_, _, entity)| set.clone().constrain("?e", entity));

        // Example: 2 bindings
        //
        //      [?e, ?a, "2022-11-06"]          => vae
        //      [42, ?a, ?v]                    => eav
        //      [?e, :rel/predecessor, ?v]      => ave
        //
        return db.vae.scan_key2_prefix("2022-11-06", [])
            .map(|(_, attr, entity)|
                set.clone()
                    .constrain("?a", attr)
                    .constrain("?e", entity)
            );

        // Example: 3 bindings
        //
        //      [?e, ?a, ?v] => eav (table scan!!)
        //
        db.eav.scan_key2_prefix([], [])
            .map(|(entity, attr, value)|
                set.clone()
                    .constrain("?e", entity)
                    .constrain("?a", attr)
                    .constrain("?v", value)
            );

        // Algorithm: Matching the stuff
        use RuleVar::*;

        enum Constraint {
            Single(Constant, Constant, Binding),
            Double(Constant, Binding, Binding),
            Triple(Binding, Binding, Binding),
        }

        // Map the binding combination to a tree and its corresponding constraint type
        let (tree, constraint) = match (self.entity, self.attribute, self.value) {
            // 1 binding
            (Binding(entity), Constant(attribute), Constant(value)) => (db.vae, Single(value, attribute, entity)),
            (Constant(entity), Constant(attribute), Binding(value)) => (db.eav, Single(entity, attribute, value)),
            (Constant(entity), Binding(attribute), Constant(value)) => unimplemented!(),

            // 2 bindings
            (Binding(entity), Binding(attribute), Constant(value)) => (db.vae, Double(value, attribute, entity)),
            (Constant(entity), Binding(attribute), Binding(value)) => (db.eav, Double(entity, attribute, value)),
            (Binding(entity), Constant(attribute), Binding(value)) => (db.ave, Double(attribute, value, entity)),

            // 3 bindings
            (Binding(entity), Binding(attribute), Binding(value)) => (db.eav, Triple(entity, attribute, value)),
        };

        // Build sets based on the constraint
        match constraint {
            Single(x, y, z) => tree.scan_key2_prefix(x, y)
                .filter(|(_, val_y, _)| val_y == y)
                .map(|(_, _, val_z)|
                    set.clone()
                        .constrain(z, val_z)
                ),

            Double(x, y, z) => tree.scan_key2_prefix(x, [])
                .map(|(_, val_y, val_z)|
                    set.clone()
                        .constrain(y, val_y)
                        .constrain(z, val_z)
                ),

            Triple(x, y, z) => tree.scan()
                .map(|(val_x, val_y, val_z)|
                    set.clone()
                        .constrain(x, val_x)
                        .constrain(y, val_y)
                        .constrain(z, val_z)
                ),
        }
    }
}
```
