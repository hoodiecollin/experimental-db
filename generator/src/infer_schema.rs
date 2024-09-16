mod components;
pub use components::*;

use std::collections::HashMap;
use syn::visit::Visit;

pub fn infer_from_file(file: &syn::File) -> Schema {
    let mut table_items = HashMap::new();

    for item in &file.items {
        if let syn::Item::Struct(item) = item {
            if item.generics != syn::Generics::default() {
                panic!("Generics are not supported: {:?}", item.ident);
            }

            table_items.insert(item.ident.to_string(), item);
        }
    }

    let table_names = table_items.keys().cloned().collect::<Vec<_>>();

    let tables = table_items
        .into_iter()
        .map(|(name, ast_item)| {
            let mut builder = TableInfoBuilder {
                other_tables: &table_names,
                name: name.clone(),
                columns: HashMap::new(),
                references: Vec::new(),
            };

            builder.visit_item_struct(ast_item);

            (name, builder.build())
        })
        .collect::<HashMap<_, _>>();

    let mut graphs = vec![];

    for (_, table) in &tables {
        if table.references.is_empty() {
            continue;
        }

        let name = table.name.clone();
        let mut graph = GraphInfo::default();
        graph.members.insert(name.clone());

        for reference in &table.references {
            let target = reference.target.clone();

            let edge = EdgeInfo {
                origin: name.clone(),
                relation: reference.clone(),
            };

            graph.members.insert(target);
            graph.edges.insert(edge);
        }

        graphs.push(graph)
    }

    // todo: handle multiple relations between the same tables

    let mut merged_graphs = vec![];
    let graphs = &mut graphs;

    while graphs.len() > 0 {
        let mut subject = graphs.pop().unwrap();
        let mut targets = vec![];

        for (i, other) in graphs.iter().enumerate() {
            if !subject.members.is_disjoint(&other.members) {
                targets.push(i);
            }
        }

        targets.reverse();

        for target in targets {
            let other = graphs.remove(target);
            subject.members.extend(other.members);
            subject.edges.extend(other.edges);
        }

        merged_graphs.push(subject);
    }

    Schema {
        tables,
        graphs: merged_graphs,
    }
}
