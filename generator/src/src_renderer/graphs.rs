use crate::infer_schema::{RelationKind, Schema};

use syn::parse_quote;

use inflection_rs::inflection;
use prettyplease::unparse;
use quote::format_ident;

pub fn render_graph_items(schema: &Schema) -> Vec<proc_macro2::TokenStream> {
    let mut graph_items = vec![];
    let graph_count = schema.graphs.len();

    for graph in &schema.graphs {
        let mut variants = vec![];
        let mut members_iter = graph.members.iter();
        let mut name = format!("{}", members_iter.next().unwrap());

        for member in members_iter {
            name = format!("{}{}", name, member);
        }

        let enum_ident = format_ident!("Connection");
        let struct_ident = format_ident!("Relationships");

        for edge in &graph.edges {
            let ident = format_ident!(
                "{}To{}",
                edge.origin,
                match edge.relation.kind {
                    RelationKind::HasOne => inflection::singularize(&edge.relation.target),
                    RelationKind::HasMany => inflection::pluralize(&edge.relation.target),
                }
            );

            variants.push(ident);
        }

        let enum_src = quote::quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum #enum_ident {
                #(#variants),*
            }
        };

        let struct_src = quote::quote! {
            pub type #struct_ident = engine::Relationships<#enum_ident>;
        };

        if graph_count == 1 {
            graph_items.push(enum_src);
            graph_items.push(struct_src);
        } else {
            let mod_ident = format_ident!("{}", inflection::underscore(name));

            graph_items.push(quote::quote! {
                pub mod #mod_ident {
                    #enum_src
                    #struct_src
                }
            });
        }
    }

    graph_items
}
