mod graphs;
mod tables;

pub use graphs::*;
pub use tables::*;

pub fn render_imports() -> proc_macro2::TokenStream {
    quote::quote! {
        extern crate flexbuffers;
        extern crate serde;

        use serde::{Deserialize, Serialize};

        use durinsbane_engine as engine;
        use derive_builder::Builder;
    }
}
