mod infer_schema;
mod src_renderer;

use infer_schema::infer_from_file;
use src_renderer::{render_graph_items, render_imports, render_table_items};

use prettyplease::unparse;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use syn::parse_quote;

fn main() -> Result<(), Box<dyn Error>> {
    // read file location from command line arguments
    let input_file = std::env::args().nth(1).expect("input file not provided");

    // read output location from command line arguments
    let output_file = std::env::args().nth(2).expect("output file not provided");

    let mut file = File::open(input_file)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let ast = syn::parse_file(&content)?;
    let schema = infer_from_file(&ast);

    let imports = render_imports();
    let core_items = render_table_items(&schema);
    let graph_items = render_graph_items(&schema);

    let mut output = File::create(output_file)?;

    let content = quote::quote! {
        #imports

        #(#core_items)*

        #(#graph_items)*
    };

    let content = unparse(&parse_quote!(#content));
    // let content = content.to_string();

    writeln!(output, "{}", content)?;

    Ok(())
}
