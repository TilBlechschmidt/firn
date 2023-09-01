use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fs::File,
    io::BufWriter,
};

use crate::{db::*, handle::Handle, query};
use serde::Serialize;

#[derive(Serialize, PartialEq, Eq, Hash)]
struct Link {
    source: String,
    target: String,
}

#[derive(Serialize, PartialEq, Eq, Hash)]
struct Node {
    id: String,
    label: String,
}

#[derive(Serialize)]
struct GraphData {
    nodes: Vec<Node>,
    links: Vec<Link>,
}

pub fn export_graph(handle: &Handle) -> Result<(), Box<dyn Error>> {
    query!(handle where (#a, #b, :attr, ?label) match [
        { #a, :attr, #b },
        { #a, :"text/label", ?label }
    ] => labelled);

    query!(handle where (#a, #b, :attr) match [
        { #a, :attr, #b }
    ] => all);

    let mut nodes = HashMap::new();
    let mut links = HashSet::new();

    for entry in all {
        let a = entry.get(&a).unwrap();

        nodes.insert(
            a.0.clone(),
            Node {
                id: a.0.clone(),
                label: a.0.clone(),
            },
        );

        if let Some(b) = entry.get(&b) {
            links.insert(Link {
                source: a.0.clone(),
                target: b.0.clone(),
            });

            nodes.insert(
                b.0.clone(),
                Node {
                    id: b.0.clone(),
                    label: b.0.clone(),
                },
            );
        }
    }

    for entry in labelled {
        let a = entry.get(&a).unwrap();
        let label = entry.get(&label).unwrap().data().to_owned();

        nodes.insert(
            a.0.clone(),
            Node {
                id: a.0.clone(),
                label,
            },
        );
    }

    let graph = GraphData {
        nodes: nodes.into_values().collect(),
        links: links.into_iter().collect(),
    };

    let file = File::create("graph.json")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &graph)?;

    Ok(())
}
